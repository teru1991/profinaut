use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::decimal::serde::deserialize_decimal_observation;
use ucel_core::{Decimal, ErrorCode, OpName, UcelError};
use ucel_transport::security::{EndpointAllowlist, SubdomainPolicy};
use ucel_transport::{
    enforce_auth_boundary, HttpRequest, RequestContext, Transport, WsConnectRequest,
};
use uuid::Uuid;

// ─── カタログ読み込み ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Catalog {
    ws_channels: Vec<CatalogWsEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogWsEntry {
    id: String,
    channel: String,
    ws_url: String,
    visibility: String,
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: String,
    pub channel: String,
    pub ws_url: String,
    pub requires_auth: bool,
}

pub fn ws_channel_specs() -> Vec<WsChannelSpec> {
    let raw = include_str!("../../../../docs/exchanges/upbit/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid upbit catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|x| WsChannelSpec {
            requires_auth: x.visibility == "private",
            id: x.id,
            channel: x.channel,
            ws_url: x.ws_url,
        })
        .collect()
}

// ─── セキュリティ：アローリスト検証 ────────────────────────────────────────────

fn upbit_allowlist() -> Result<EndpointAllowlist, UcelError> {
    EndpointAllowlist::new(["upbit.com"], SubdomainPolicy::AllowSubdomains)
}

fn validate_ws_url(raw: &str) -> Result<(), UcelError> {
    upbit_allowlist()?.validate_https_wss(raw).map(|_| ())
}

fn validate_http_base_url(raw: &str) -> Result<(), UcelError> {
    upbit_allowlist()?.validate_https_wss(raw).map(|_| ())
}

// ─── パスレンダリング ──────────────────────────────────────────────────────────

fn render_path_and_body(
    method: &str,
    path: &str,
    body: Option<Bytes>,
) -> Result<(String, Option<Bytes>), UcelError> {
    let rendered = path.replace("{unit}", "1");
    if method.eq_ignore_ascii_case("GET") {
        Ok((rendered, None))
    } else {
        Ok((rendered, body))
    }
}

// ─── WebSocket：メトリクス・フレーム・メッセージ ──────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct WsAdapterMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_drops_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpbitSubscribeFrame {
    pub ticket: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker {
        code: String,
        trade_price: Decimal,
    },
    Trade {
        code: String,
        trade_price: Decimal,
        trade_volume: Decimal,
    },
    Orderbook {
        code: String,
        total_ask_size: Option<Decimal>,
        total_bid_size: Option<Decimal>,
    },
    Candle {
        code: String,
        trade_price: Decimal,
    },
    SubscriptionList {
        channels: Vec<String>,
    },
    MyOrder {
        code: Option<String>,
        state: Option<String>,
    },
    MyAsset {
        currency: Option<String>,
        balance: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum UpbitMessage {
    #[serde(rename = "ticker")]
    Ticker {
        code: String,
        #[serde(deserialize_with = "deserialize_decimal_observation")]
        trade_price: Decimal,
    },
    #[serde(rename = "trade")]
    Trade {
        code: String,
        #[serde(deserialize_with = "deserialize_decimal_observation")]
        trade_price: Decimal,
        #[serde(deserialize_with = "deserialize_decimal_observation")]
        trade_volume: Decimal,
    },
    #[serde(rename = "orderbook")]
    Orderbook {
        code: String,
        #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
        total_ask_size: Option<Decimal>,
        #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
        total_bid_size: Option<Decimal>,
        #[serde(rename = "timestamp")]
        _timestamp: Option<u64>,
    },
    #[serde(rename = "candle")]
    Candle {
        code: String,
        #[serde(deserialize_with = "deserialize_decimal_observation")]
        trade_price: Decimal,
    },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions { codes: Vec<String> },
    #[serde(rename = "myOrder")]
    MyOrder {
        code: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "myAsset")]
    MyAsset {
        currency: Option<String>,
        balance: Option<String>,
    },
}

fn deserialize_opt_decimal_observation<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<serde_json::Value>::deserialize(deserializer)?.map_or(Ok(None), |v| {
        let raw = v.to_string();
        let d = raw
            .trim_matches('"')
            .parse::<Decimal>()
            .map_err(serde::de::Error::custom)?;
        let policy = ucel_core::decimal::DecimalPolicy::for_observation_relaxed();
        policy
            .guard()
            .validate(d)
            .map(Some)
            .map_err(serde::de::Error::custom)
    })
}

pub fn normalize_ws_message(raw: &str) -> Result<MarketEvent, UcelError> {
    let msg: UpbitMessage = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("typed ws parse error: {e}")))?;
    Ok(match msg {
        UpbitMessage::Ticker { code, trade_price } => MarketEvent::Ticker { code, trade_price },
        UpbitMessage::Trade {
            code,
            trade_price,
            trade_volume,
        } => MarketEvent::Trade {
            code,
            trade_price,
            trade_volume,
        },
        UpbitMessage::Orderbook {
            code,
            total_ask_size,
            total_bid_size,
            ..
        } => MarketEvent::Orderbook {
            code,
            total_ask_size,
            total_bid_size,
        },
        UpbitMessage::Candle { code, trade_price } => MarketEvent::Candle { code, trade_price },
        UpbitMessage::ListSubscriptions { codes } => {
            MarketEvent::SubscriptionList { channels: codes }
        }
        UpbitMessage::MyOrder { code, state } => MarketEvent::MyOrder { code, state },
        UpbitMessage::MyAsset { currency, balance } => MarketEvent::MyAsset { currency, balance },
    })
}

// ─── オーダーブック同期 ────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct OrderbookSync {
    pub last_ts: Option<u64>,
    pub degraded: bool,
    pub book: BTreeMap<String, f64>,
}

impl OrderbookSync {
    pub fn apply_snapshot(&mut self, ts: u64) {
        self.last_ts = Some(ts);
        self.degraded = false;
    }

    pub fn apply_delta(
        &mut self,
        ts: u64,
        metrics: &mut WsAdapterMetrics,
    ) -> Result<(), UcelError> {
        match self.last_ts {
            Some(prev) if ts <= prev => {
                self.degraded = true;
                metrics.ws_orderbook_gap_total += 1;
                metrics.ws_orderbook_resync_total += 1;
                Err(UcelError::new(
                    ErrorCode::Desync,
                    "gap/mismatch detected, immediate resync",
                ))
            }
            Some(_) => {
                self.last_ts = Some(ts);
                Ok(())
            }
            None => {
                self.degraded = true;
                metrics.ws_orderbook_gap_total += 1;
                metrics.ws_orderbook_resync_total += 1;
                Err(UcelError::new(ErrorCode::Desync, "delta before snapshot"))
            }
        }
    }

    pub fn resync(&mut self, snapshot_ts: u64, metrics: &mut WsAdapterMetrics) {
        let was_degraded = self.degraded;
        self.apply_snapshot(snapshot_ts);
        if was_degraded {
            metrics.ws_orderbook_recovered_total += 1;
        }
    }

    pub fn mark_recovered(&mut self, metrics: &mut WsAdapterMetrics) {
        if self.degraded {
            self.degraded = false;
            metrics.ws_orderbook_recovered_total += 1;
        }
    }
}

// ─── バックプレッシャーキュー ──────────────────────────────────────────────────

pub struct BackpressureQueue {
    tx: mpsc::Sender<MarketEvent>,
    rx: mpsc::Receiver<MarketEvent>,
}

impl BackpressureQueue {
    pub fn with_capacity(cap: usize) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx }
    }

    pub fn try_push(&self, ev: MarketEvent, metrics: &mut WsAdapterMetrics) {
        if self.tx.try_send(ev).is_err() {
            metrics.ws_backpressure_drops_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<MarketEvent> {
        self.rx.recv().await
    }
}

// ─── WS アダプタ ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct UpbitWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: WsAdapterMetrics,
}

impl UpbitWsAdapter {
    pub fn build_subscribe(
        endpoint_id: &str,
        code: &str,
        key_id: Option<&str>,
    ) -> Result<UpbitSubscribeFrame, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private ws endpoint requires auth",
            ));
        }
        if spec.requires_auth {
            info!(
                target: "upbit.auth",
                key_id = %key_id.unwrap_or(""),
                "private ws subscribe preflight passed"
            );
        }
        Ok(UpbitSubscribeFrame {
            ticket: "ucel-upbit".into(),
            channel_type: spec.channel,
            codes: vec![code.into()],
        })
    }

    pub fn build_unsubscribe(
        endpoint_id: &str,
        code: &str,
    ) -> Result<UpbitSubscribeFrame, UcelError> {
        Self::build_subscribe(endpoint_id, code, Some("dummy"))
    }

    pub fn subscribe_once(&mut self, endpoint_id: &str, code: &str) -> bool {
        self.subscriptions.insert(format!("{endpoint_id}:{code}"))
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(
        &mut self,
        transport: &T,
    ) -> Result<usize, UcelError> {
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "upbit".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(
                WsConnectRequest {
                    url: {
                        let u = "wss://api.upbit.com/websocket/v1";
                        validate_ws_url(u)?;
                        u.into()
                    },
                },
                ctx,
            )
            .await?;
        self.metrics.ws_reconnect_total += 1;
        self.metrics.ws_resubscribe_total += self.subscriptions.len() as u64;
        Ok(self.subscriptions.len())
    }
}

// ─── シークレットスクラブ ──────────────────────────────────────────────────────

pub fn scrub_secrets(line: &str) -> String {
    line.split_whitespace()
        .map(|x| {
            if x.starts_with("api_key=") || x.starts_with("access_key=") {
                "api_key=***".to_string()
            } else if x.starts_with("api_secret=") || x.starts_with("secret_key=") {
                "api_secret=***".to_string()
            } else if x.starts_with("authorization=") || x.starts_with("Authorization=") {
                "authorization=***".to_string()
            } else {
                x.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ─── REST アダプタ ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct UpbitRestAdapter {
    endpoints: Vec<EndpointSpec>,
}

impl Default for UpbitRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UpbitRestAdapter {
    pub fn new() -> Self {
        Self {
            endpoints: load_endpoint_specs(),
        }
    }

    pub fn endpoint_specs(&self) -> &[EndpointSpec] {
        &self.endpoints
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<UpbitRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|entry| entry.id == endpoint_id)
            .ok_or_else(|| {
                UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unknown endpoint: {endpoint_id}"),
                )
            })?;

        validate_http_base_url(&spec.base_url)?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: op_for_rest(endpoint_id),
            venue: "upbit".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let (rendered_path, send_body) = render_path_and_body(&spec.method, &spec.path, body)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}{}", spec.base_url, rendered_path),
            body: send_body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_upbit_http_error(response.status, &response.body));
        }

        parse_upbit_response(endpoint_id, &response.body)
    }
}

// ─── レスポンス型 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum UpbitRestResponse {
    Markets(Vec<UpbitMarket>),
    Tickers(Vec<UpbitTicker>),
    Trades(Vec<UpbitTrade>),
    Orderbook(Vec<UpbitOrderbook>),
    Candles(Vec<UpbitCandle>),
    Accounts(Vec<UpbitAccount>),
    CreateOrder(UpbitOrder),
    CancelOrder(UpbitOrder),
    OpenOrders(Vec<UpbitOrder>),
    ClosedOrders(Vec<UpbitOrder>),
    OrderChance(UpbitOrderChance),
    Withdraws(Vec<UpbitWithdraw>),
    WithdrawCoin(UpbitWithdraw),
    Deposits(Vec<UpbitDeposit>),
    DepositAddress(UpbitDepositAddress),
    TravelRuleVasps(Vec<UpbitVasp>),
    WalletStatus(Vec<UpbitWalletStatus>),
    ApiKeys(Vec<UpbitApiKey>),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitMarket {
    pub market: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitTicker {
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub trade_price: Decimal,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitTrade {
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub trade_price: Decimal,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub trade_volume: Decimal,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitOrderbook {
    pub market: String,
    pub orderbook_units: Vec<UpbitOrderbookUnit>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitOrderbookUnit {
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub ask_price: Decimal,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub bid_price: Decimal,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub ask_size: Decimal,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub bid_size: Decimal,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitCandle {
    #[serde(default)]
    pub candle_date_time_utc: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub opening_price: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub trade_price: Decimal,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitAccount {
    pub currency: String,
    pub balance: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitOrder {
    pub uuid: String,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitOrderChance {
    pub market: UpbitMarketInfo,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitMarketInfo {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitWithdraw {
    pub currency: String,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitDeposit {
    pub currency: String,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitDepositAddress {
    pub currency: String,
    pub deposit_address: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitVasp {
    pub vasp_name: String,
    pub vasp_code: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitWalletStatus {
    pub currency: String,
    pub wallet_state: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpbitApiKey {
    pub access_key: String,
    pub expire_at: String,
}

// ─── レスポンスパーサ ──────────────────────────────────────────────────────────

fn parse_upbit_response(endpoint_id: &str, body: &[u8]) -> Result<UpbitRestResponse, UcelError> {
    match endpoint_id {
        "quotation.public.rest.markets.list" => {
            Ok(UpbitRestResponse::Markets(parse_json(body)?))
        }
        "quotation.public.rest.ticker.pairs" => {
            Ok(UpbitRestResponse::Tickers(parse_json(body)?))
        }
        "quotation.public.rest.trades.recent" => {
            Ok(UpbitRestResponse::Trades(parse_json(body)?))
        }
        "quotation.public.rest.orderbook.snapshot" => {
            Ok(UpbitRestResponse::Orderbook(parse_json(body)?))
        }
        "quotation.public.rest.candles.minutes"
        | "quotation.public.rest.candles.days"
        | "quotation.public.rest.candles.weeks"
        | "quotation.public.rest.candles.months"
        | "quotation.public.rest.candles.years" => {
            Ok(UpbitRestResponse::Candles(parse_json(body)?))
        }
        "exchange.private.rest.accounts.list" => {
            Ok(UpbitRestResponse::Accounts(parse_json(body)?))
        }
        "exchange.private.rest.orders.create" => {
            Ok(UpbitRestResponse::CreateOrder(parse_json(body)?))
        }
        "exchange.private.rest.orders.cancel" => {
            Ok(UpbitRestResponse::CancelOrder(parse_json(body)?))
        }
        "exchange.private.rest.orders.open" => {
            Ok(UpbitRestResponse::OpenOrders(parse_json(body)?))
        }
        "exchange.private.rest.orders.closed" => {
            Ok(UpbitRestResponse::ClosedOrders(parse_json(body)?))
        }
        "exchange.private.rest.orders.chance" => {
            Ok(UpbitRestResponse::OrderChance(parse_json(body)?))
        }
        "exchange.private.rest.withdraws.list" => {
            Ok(UpbitRestResponse::Withdraws(parse_json(body)?))
        }
        "exchange.private.rest.withdraws.coin" => {
            Ok(UpbitRestResponse::WithdrawCoin(parse_json(body)?))
        }
        "exchange.private.rest.deposits.list" => {
            Ok(UpbitRestResponse::Deposits(parse_json(body)?))
        }
        "exchange.private.rest.deposits.coin_address" => {
            Ok(UpbitRestResponse::DepositAddress(parse_json(body)?))
        }
        "exchange.private.rest.travel_rule.vasps" => {
            Ok(UpbitRestResponse::TravelRuleVasps(parse_json(body)?))
        }
        "exchange.private.rest.status.wallet" => {
            Ok(UpbitRestResponse::WalletStatus(parse_json(body)?))
        }
        "exchange.private.rest.api_keys.list" => {
            Ok(UpbitRestResponse::ApiKeys(parse_json(body)?))
        }
        other => Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("unmapped response parser for endpoint: {other}"),
        )),
    }
}

fn parse_json<T: DeserializeOwned>(body: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(body)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

// ─── HTTP エラーマッピング ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct UpbitErrorEnvelope {
    #[serde(default)]
    error: Option<UpbitErrorBody>,
    #[serde(default)]
    retry_after_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpbitErrorBody {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

fn map_upbit_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = serde_json::from_slice::<UpbitErrorEnvelope>(body)
            .ok()
            .and_then(|env| env.retry_after_ms);
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream error");
    }

    let parsed = serde_json::from_slice::<UpbitErrorEnvelope>(body).ok();
    let (name, message) = parsed
        .and_then(|x| x.error)
        .map(|e| {
            (
                e.name.unwrap_or_else(|| "unknown".to_string()),
                e.message.unwrap_or_else(|| "unknown".to_string()),
            )
        })
        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

    let msg = format!("upbit http error: status={status} name={name} message={message}");

    match status {
        401 | 403 => UcelError::new(ErrorCode::MissingAuth, msg),
        404 => UcelError::new(ErrorCode::NotSupported, msg),
        _ => UcelError::new(ErrorCode::Internal, msg),
    }
}

// ─── カタログ駆動のエンドポイント読み込み ─────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RestCatalog {
    rest_endpoints: Vec<RestCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct RestCatalogEntry {
    id: String,
    method: String,
    base_url: String,
    path: String,
    visibility: String,
    auth: RestCatalogAuth,
}

#[derive(Debug, Deserialize)]
struct RestCatalogAuth {
    #[serde(rename = "type")]
    auth_type: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/upbit/catalog.json");
    let catalog: RestCatalog = serde_json::from_str(raw).expect("valid upbit catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            base_url: entry.base_url,
            path: entry.path,
            requires_auth: entry.auth.auth_type != "none"
                || entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

fn op_for_rest(endpoint_id: &str) -> OpName {
    match endpoint_id {
        "quotation.public.rest.ticker.pairs" => OpName::FetchTicker,
        "quotation.public.rest.trades.recent" => OpName::FetchTrades,
        "quotation.public.rest.orderbook.snapshot" => OpName::FetchOrderbookSnapshot,
        "quotation.public.rest.candles.minutes"
        | "quotation.public.rest.candles.days"
        | "quotation.public.rest.candles.weeks"
        | "quotation.public.rest.candles.months"
        | "quotation.public.rest.candles.years" => OpName::FetchKlines,
        "exchange.private.rest.orders.create" => OpName::PlaceOrder,
        "exchange.private.rest.orders.cancel" => OpName::CancelOrder,
        "exchange.private.rest.orders.open" => OpName::FetchOpenOrders,
        "exchange.private.rest.accounts.list" => OpName::FetchBalances,
        _ => OpName::FetchStatus,
    }
}

// ─── テスト ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use ucel_transport::{HttpRequest, HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        ws_connects: AtomicUsize,
    }
    impl SpyTransport {
        fn new() -> Self {
            Self {
                ws_connects: AtomicUsize::new(0),
            }
        }
    }
    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Err(UcelError::new(ErrorCode::NotSupported, "unused"))
        }
        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_connects.fetch_add(1, Ordering::Relaxed);
            Ok(WsStream { connected: true })
        }
    }

    #[test]
    fn all_ws_catalog_rows_build_subscribe_unsubscribe() {
        for spec in ws_channel_specs() {
            let key = if spec.requires_auth { Some("kid") } else { None };
            assert_eq!(
                UpbitWsAdapter::build_subscribe(&spec.id, "KRW-BTC", key)
                    .unwrap()
                    .channel_type,
                spec.channel
            );
            assert_eq!(
                UpbitWsAdapter::build_unsubscribe(&spec.id, "KRW-BTC")
                    .unwrap()
                    .channel_type,
                spec.channel
            );
        }
    }

    #[test]
    fn private_preflight_reject_no_connect() {
        let err =
            UpbitWsAdapter::build_subscribe("exchange.private.ws.myasset.stream", "KRW-BTC", None)
                .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_idempotent() {
        let spy = SpyTransport::new();
        let mut ws = UpbitWsAdapter::default();
        assert!(ws.subscribe_once("quotation.public.ws.trade.stream", "KRW-BTC"));
        assert!(!ws.subscribe_once("quotation.public.ws.trade.stream", "KRW-BTC"));
        let restored = ws.reconnect_and_resubscribe(&spy).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 1);
        assert_eq!(spy.ws_connects.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn typed_deserialize_and_normalize() {
        let t =
            normalize_ws_message(r#"{"type":"ticker","code":"KRW-BTC","trade_price":1.0}"#)
                .unwrap();
        assert!(matches!(t, MarketEvent::Ticker { .. }));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_and_metrics() {
        let mut q = BackpressureQueue::with_capacity(1);
        let mut m = WsAdapterMetrics::default();
        q.try_push(
            MarketEvent::Candle {
                code: "KRW-BTC".into(),
                trade_price: "1.0".parse::<Decimal>().unwrap(),
            },
            &mut m,
        );
        q.try_push(
            MarketEvent::Candle {
                code: "KRW-ETH".into(),
                trade_price: "2.0".parse::<Decimal>().unwrap(),
            },
            &mut m,
        );
        assert_eq!(m.ws_backpressure_drops_total, 1);
        assert!(
            matches!(q.recv().await.unwrap(), MarketEvent::Candle { code, .. } if code == "KRW-BTC")
        );
    }

    #[test]
    fn orderbook_gap_resync_recovered() {
        let mut sync = OrderbookSync::default();
        let mut m = WsAdapterMetrics::default();
        sync.apply_snapshot(100);
        assert!(sync.apply_delta(100, &mut m).is_err());
        assert!(sync.degraded);
        sync.resync(101, &mut m);
        assert!(!sync.degraded);
        assert_eq!(m.ws_orderbook_gap_total, 1);
        assert_eq!(m.ws_orderbook_resync_total, 1);
        assert_eq!(m.ws_orderbook_recovered_total, 1);
    }

    #[test]
    fn duplicate_and_out_of_order_policy_forces_resync() {
        let mut sync = OrderbookSync::default();
        let mut m = WsAdapterMetrics::default();
        sync.apply_snapshot(10);
        assert!(sync.apply_delta(9, &mut m).is_err());
        assert!(sync.degraded);
    }

    #[test]
    fn no_secret_leak() {
        let line = "key_id=alpha api_key=hello api_secret=world";
        let scrubbed = scrub_secrets(line);
        assert!(scrubbed.contains("key_id=alpha"));
        assert!(!scrubbed.contains("hello"));
        assert!(!scrubbed.contains("world"));
    }

    // NOTE: このテストを有効にするには Cargo.toml の [dev-dependencies] に
    //   ucel_testkit = { path = "..." }  （または workspace = true）
    // を追加してください。
    #[test]
    fn strict_gate_enabled_and_zero_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../coverage/upbit.yaml");
        let manifest = ucel_testkit::load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = ucel_testkit::evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict gate requires zero gaps: {gaps:?}");
    }
}

pub mod channels;
pub mod symbols;
pub mod ws_manager;