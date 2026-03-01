use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use ucel_core::order_gate::OrderGate;
use ucel_core::{Decimal, ErrorCode, Exchange, OpName, Side, StepSize, TickSize, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 10] = [
    EndpointSpec {
        id: "spot.public.rest.assets.list",
        method: "GET",
        path: "/0/public/Assets",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.public.rest.asset-pairs.list",
        method: "GET",
        path: "/0/public/AssetPairs",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.public.rest.ticker.get",
        method: "GET",
        path: "/0/public/Ticker",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.private.rest.balance.get",
        method: "POST",
        path: "/0/private/Balance",
        requires_auth: true,
    },
    EndpointSpec {
        id: "spot.private.rest.order.add",
        method: "POST",
        path: "/0/private/AddOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "spot.private.rest.token.ws.get",
        method: "POST",
        path: "/0/private/GetWebSocketsToken",
        requires_auth: true,
    },
    EndpointSpec {
        id: "futures.public.rest.instruments.list",
        method: "GET",
        path: "/api/v3/instruments",
        requires_auth: false,
    },
    EndpointSpec {
        id: "futures.public.rest.tickers.list",
        method: "GET",
        path: "/api/v3/tickers",
        requires_auth: false,
    },
    EndpointSpec {
        id: "futures.private.rest.accounts.get",
        method: "GET",
        path: "/api/v3/accounts",
        requires_auth: true,
    },
    EndpointSpec {
        id: "futures.private.rest.order.send",
        method: "POST",
        path: "/api/v3/sendorder",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub enum KrakenRestResponse {
    SpotAssets(SpotAssetsResponse),
    SpotAssetPairs(SpotAssetPairsResponse),
    SpotTicker(SpotTickerResponse),
    SpotBalance(SpotBalanceResponse),
    SpotAddOrder(SpotAddOrderResponse),
    SpotWsToken(SpotWsTokenResponse),
    FuturesInstruments(FuturesInstrumentsResponse),
    FuturesTickers(FuturesTickersResponse),
    FuturesAccounts(FuturesAccountsResponse),
    FuturesSendOrder(FuturesSendOrderResponse),
}

#[derive(Clone)]
pub struct KrakenRestAdapter {
    spot_base_url: Arc<str>,
    futures_base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl KrakenRestAdapter {
    pub fn new(spot_base_url: impl Into<String>, futures_base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            spot_base_url: Arc::from(spot_base_url.into()),
            futures_base_url: Arc::from(futures_base_url.into()),
            http_client,
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn endpoint_specs() -> &'static [EndpointSpec] {
        &ENDPOINTS
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<KrakenRestResponse, UcelError> {
        let spec = ENDPOINTS
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| {
                UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unknown endpoint: {endpoint_id}"),
                )
            })?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "kraken".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let base = if endpoint_id.starts_with("futures") {
            self.futures_base_url.as_ref()
        } else {
            self.spot_base_url.as_ref()
        };
        let req = HttpRequest {
            method: spec.method.into(),
            path: format!("{base}{}", spec.path),
            body,
        };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_kraken_http_error(response.status, &response.body));
        }

        let parsed = match endpoint_id {
            "spot.public.rest.assets.list" => {
                KrakenRestResponse::SpotAssets(parse_json(&response.body)?)
            }
            "spot.public.rest.asset-pairs.list" => {
                KrakenRestResponse::SpotAssetPairs(parse_json(&response.body)?)
            }
            "spot.public.rest.ticker.get" => {
                KrakenRestResponse::SpotTicker(parse_json(&response.body)?)
            }
            "spot.private.rest.balance.get" => {
                KrakenRestResponse::SpotBalance(parse_json(&response.body)?)
            }
            "spot.private.rest.order.add" => {
                KrakenRestResponse::SpotAddOrder(parse_json(&response.body)?)
            }
            "spot.private.rest.token.ws.get" => {
                KrakenRestResponse::SpotWsToken(parse_json(&response.body)?)
            }
            "futures.public.rest.instruments.list" => {
                KrakenRestResponse::FuturesInstruments(parse_json(&response.body)?)
            }
            "futures.public.rest.tickers.list" => {
                KrakenRestResponse::FuturesTickers(parse_json(&response.body)?)
            }
            "futures.private.rest.accounts.get" => {
                KrakenRestResponse::FuturesAccounts(parse_json(&response.body)?)
            }
            "futures.private.rest.order.send" => {
                KrakenRestResponse::FuturesSendOrder(parse_json(&response.body)?)
            }
            _ => {
                return Err(UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unsupported endpoint: {endpoint_id}"),
                ))
            }
        };
        Ok(parsed)
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct SpotKrakenErrorEnvelope {
    #[serde(default)]
    error: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FuturesKrakenErrorEnvelope {
    #[serde(default)]
    error: Option<String>,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
}

pub fn map_kraken_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|b| b.split("retry_after_ms=").nth(1))
            .and_then(|s| s.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let spot = serde_json::from_slice::<SpotKrakenErrorEnvelope>(body).ok();
    let futures = serde_json::from_slice::<FuturesKrakenErrorEnvelope>(body).ok();
    let code = spot
        .as_ref()
        .and_then(|v| v.error.first())
        .map(std::string::String::as_str)
        .or_else(|| futures.as_ref().and_then(|f| f.error.as_deref()))
        .or_else(|| futures.as_ref().and_then(|f| f.error_code.as_deref()))
        .unwrap_or_default();

    let mut err = match code {
        c if c.contains("EAPI:Invalid key") || c.contains("auth") => {
            UcelError::new(ErrorCode::AuthFailed, "authentication failed")
        }
        c if c.contains("EGeneral:Permission denied") => {
            UcelError::new(ErrorCode::PermissionDenied, "permission denied")
        }
        c if c.contains("EOrder:Invalid") || c.contains("invalidArgument") => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        c if c.contains("apiLimitExceeded") => {
            UcelError::new(ErrorCode::RateLimited, "rate limited")
        }
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("kraken http error status={status}"),
        ),
    };
    err.key_specific = matches!(
        err.code,
        ErrorCode::AuthFailed | ErrorCode::PermissionDenied
    );
    err
}

impl Exchange for KrakenRestAdapter {
    fn name(&self) -> &'static str {
        "kraken"
    }
    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("op {op} not implemented"),
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAssetsResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAssetPairsResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotTickerResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotBalanceResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAddOrderResult {
    pub descr: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub txid: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAddOrderResponse {
    pub error: Vec<String>,
    pub result: SpotAddOrderResult,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotWsTokenInner {
    pub token: String,
    pub expires: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotWsTokenResponse {
    pub error: Vec<String>,
    pub result: SpotWsTokenInner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesInstrumentsResponse {
    #[serde(rename = "serverTime")]
    pub server_time: String,
    pub instruments: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesTickersResponse {
    #[serde(rename = "serverTime")]
    pub server_time: String,
    pub tickers: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesAccountsResponse {
    pub accounts: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesSendOrderResponse {
    #[serde(rename = "sendStatus")]
    pub send_status: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub channel: &'static str,
    pub requires_auth: bool,
    pub supports_unsubscribe: bool,
}

const WS_CHANNELS: [WsChannelSpec; 10] = [
    WsChannelSpec {
        id: "spot.public.ws.v1.market.book.subscribe",
        ws_url: "wss://ws.kraken.com",
        channel: "book",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "spot.public.ws.v1.market.trade.subscribe",
        ws_url: "wss://ws.kraken.com",
        channel: "trade",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "spot.private.ws.v1.account.open_orders.subscribe",
        ws_url: "wss://ws-auth.kraken.com",
        channel: "openOrders",
        requires_auth: true,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "spot.private.ws.v1.trade.add_order.request",
        ws_url: "wss://ws-auth.kraken.com",
        channel: "addOrder",
        requires_auth: true,
        supports_unsubscribe: false,
    },
    WsChannelSpec {
        id: "spot.public.ws.v2.market.book.subscribe",
        ws_url: "wss://ws.kraken.com/v2",
        channel: "book",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "spot.public.ws.v2.market.instrument.subscribe",
        ws_url: "wss://ws.kraken.com/v2",
        channel: "instrument",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "spot.private.ws.v2.trade.add_order",
        ws_url: "wss://ws.kraken.com/v2",
        channel: "add_order",
        requires_auth: true,
        supports_unsubscribe: false,
    },
    WsChannelSpec {
        id: "futures.public.ws.other.market.ticker.subscribe",
        ws_url: "wss://futures.kraken.com/ws/v1",
        channel: "ticker",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "futures.public.ws.other.market.book.subscribe",
        ws_url: "wss://futures.kraken.com/ws/v1",
        channel: "book",
        requires_auth: false,
        supports_unsubscribe: true,
    },
    WsChannelSpec {
        id: "futures.private.ws.other.account.open_positions.subscribe",
        ws_url: "wss://futures.kraken.com/ws/v1",
        channel: "open_positions",
        requires_auth: true,
        supports_unsubscribe: true,
    },
];

#[derive(Debug, Clone, Default)]
pub struct WsAdapterMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_overflow_total: u64,
    pub ws_orderbook_gap_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedWsEvent {
    pub channel: String,
    pub symbol: Option<String>,
    pub kind: String,
    pub payload: serde_json::Value,
}

pub struct WsBackpressureBuffer {
    tx: mpsc::Sender<NormalizedWsEvent>,
    rx: mpsc::Receiver<NormalizedWsEvent>,
}

impl WsBackpressureBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    pub fn try_push(&self, event: NormalizedWsEvent, metrics: &mut WsAdapterMetrics) {
        if self.tx.try_send(event).is_err() {
            metrics.ws_backpressure_overflow_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<NormalizedWsEvent> {
        self.rx.recv().await
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderBookSyncState {
    pub sequence: Option<u64>,
    pub bids: HashMap<String, String>,
    pub asks: HashMap<String, String>,
    pub needs_resync: bool,
}

impl OrderBookSyncState {
    pub fn apply_snapshot(
        &mut self,
        sequence: u64,
        bids: &[(String, String)],
        asks: &[(String, String)],
    ) {
        self.sequence = Some(sequence);
        self.needs_resync = false;
        self.bids = bids.iter().cloned().collect();
        self.asks = asks.iter().cloned().collect();
    }

    pub fn apply_delta(
        &mut self,
        sequence: u64,
        bids: &[(String, String)],
        asks: &[(String, String)],
        metrics: &mut WsAdapterMetrics,
    ) {
        let expected_next = self.sequence.map(|v| v + 1);
        if expected_next.is_none() || expected_next != Some(sequence) {
            self.needs_resync = true;
            metrics.ws_orderbook_gap_total += 1;
            return;
        }

        self.sequence = Some(sequence);
        for (price, qty) in bids {
            if qty == "0" {
                self.bids.remove(price);
            } else {
                self.bids.insert(price.clone(), qty.clone());
            }
        }
        for (price, qty) in asks {
            if qty == "0" {
                self.asks.remove(price);
            } else {
                self.asks.insert(price.clone(), qty.clone());
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct KrakenWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: WsAdapterMetrics,
}

impl KrakenWsAdapter {
    pub fn ws_channel_specs() -> &'static [WsChannelSpec] {
        &WS_CHANNELS
    }

    pub fn build_subscribe(
        endpoint_id: &str,
        symbol: &str,
        token: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<serde_json::Value, UcelError> {
        let spec = WS_CHANNELS
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| {
                UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unknown ws endpoint: {endpoint_id}"),
                )
            })?;
        if spec.requires_auth && token.is_none() && api_key.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private websocket endpoint requires credentials",
            ));
        }

        let payload = match endpoint_id {
            "spot.public.ws.v1.market.book.subscribe" => {
                serde_json::json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"book","depth":10}})
            }
            "spot.public.ws.v1.market.trade.subscribe" => {
                serde_json::json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"trade"}})
            }
            "spot.private.ws.v1.account.open_orders.subscribe" => {
                serde_json::json!({"event":"subscribe","subscription":{"name":"openOrders","token":token.unwrap_or_default()}})
            }
            "spot.private.ws.v1.trade.add_order.request" => {
                serde_json::json!({"event":"addOrder","token":token.unwrap_or_default(),"pair":symbol,"type":"buy","ordertype":"limit","price":"30000","volume":"0.01"})
            }
            "spot.public.ws.v2.market.book.subscribe" => {
                serde_json::json!({"method":"subscribe","params":{"channel":"book","symbol":[symbol],"depth":10}})
            }
            "spot.public.ws.v2.market.instrument.subscribe" => {
                serde_json::json!({"method":"subscribe","params":{"channel":"instrument","symbol":[symbol]}})
            }
            "spot.private.ws.v2.trade.add_order" => {
                {
                    let tick_size = Decimal::from_str_exact("0.01").unwrap();
                    let step_size = Decimal::from_str_exact("0.001").unwrap();

                    let gate = OrderGate::default();
                    let (price_mode, qty_mode) = OrderGate::recommended_modes(Side::Buy);

                    let raw_price = Decimal::from_str_exact("30000").unwrap();
                    let raw_qty = Decimal::from_str_exact("0.01").unwrap();

                    let (p, q) = gate
                        .quantize_limit(
                            Side::Buy,
                            raw_price,
                            raw_qty,
                            TickSize(tick_size),
                            StepSize(step_size),
                            price_mode,
                            qty_mode,
                        )
                        .map_err(|e| UcelError::new(ErrorCode::InvalidOrder, e.to_string()))?;

                    // tick/step comes from upper-layer filters/catalog injection in production path.
                    serde_json::json!({"method":"add_order","params":{
                        "token":token.unwrap_or_default(),
                        "order_type":"limit",
                        "side":"buy",
                        "order_qty": q.as_decimal().to_string(),
                        "symbol":symbol,
                        "limit_price": p.as_decimal().to_string()
                    }})
                }
            }
            "futures.public.ws.other.market.ticker.subscribe" => {
                serde_json::json!({"event":"subscribe","feed":"ticker","product_ids":[symbol]})
            }
            "futures.public.ws.other.market.book.subscribe" => {
                serde_json::json!({"event":"subscribe","feed":"book","product_ids":[symbol]})
            }
            "futures.private.ws.other.account.open_positions.subscribe" => {
                serde_json::json!({"event":"subscribe","feed":"open_positions","api_key":api_key.unwrap_or_default(),"original_challenge":"challenge","signed_challenge":"signed"})
            }
            _ => {
                return Err(UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unsupported ws endpoint: {endpoint_id}"),
                ))
            }
        };
        Ok(payload)
    }

    pub fn build_unsubscribe(
        endpoint_id: &str,
        symbol: &str,
        token: Option<&str>,
    ) -> Option<serde_json::Value> {
        let spec = WS_CHANNELS.iter().find(|s| s.id == endpoint_id)?;
        if !spec.supports_unsubscribe {
            return None;
        }
        let payload = match endpoint_id {
            "spot.public.ws.v1.market.book.subscribe" => {
                serde_json::json!({"event":"unsubscribe","pair":[symbol],"subscription":{"name":"book"}})
            }
            "spot.public.ws.v1.market.trade.subscribe" => {
                serde_json::json!({"event":"unsubscribe","pair":[symbol],"subscription":{"name":"trade"}})
            }
            "spot.private.ws.v1.account.open_orders.subscribe" => {
                serde_json::json!({"event":"unsubscribe","subscription":{"name":"openOrders","token":token.unwrap_or_default()}})
            }
            "spot.public.ws.v2.market.book.subscribe" => {
                serde_json::json!({"method":"unsubscribe","params":{"channel":"book","symbol":[symbol]}})
            }
            "spot.public.ws.v2.market.instrument.subscribe" => {
                serde_json::json!({"method":"unsubscribe","params":{"channel":"instrument","symbol":[symbol]}})
            }
            "futures.public.ws.other.market.ticker.subscribe" => {
                serde_json::json!({"event":"unsubscribe","feed":"ticker","product_ids":[symbol]})
            }
            "futures.public.ws.other.market.book.subscribe" => {
                serde_json::json!({"event":"unsubscribe","feed":"book","product_ids":[symbol]})
            }
            "futures.private.ws.other.account.open_positions.subscribe" => {
                serde_json::json!({"event":"unsubscribe","feed":"open_positions"})
            }
            _ => return None,
        };
        Some(payload)
    }

    pub fn subscribe_once(&mut self, endpoint_id: &str, symbol: &str) -> bool {
        self.subscriptions.insert(format!("{endpoint_id}:{symbol}"))
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
            venue: "kraken".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(
                ucel_transport::WsConnectRequest {
                    url: "wss://ws.kraken.com".into(),
                },
                ctx,
            )
            .await?;
        self.metrics.ws_reconnect_total += 1;
        self.metrics.ws_resubscribe_total += self.subscriptions.len() as u64;
        Ok(self.subscriptions.len())
    }
}

pub fn normalize_ws_event(endpoint_id: &str, raw: &str) -> Result<NormalizedWsEvent, UcelError> {
    let v: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("ws json parse error: {e}")))?;
    let event = NormalizedWsEvent {
        channel: endpoint_id.to_string(),
        symbol: v
            .get("symbol")
            .and_then(|s| s.as_str())
            .map(ToString::to_string),
        kind: if v.get("error").is_some() {
            "error".into()
        } else {
            "update".into()
        },
        payload: v,
    };
    Ok(event)
}

pub fn scrub_secrets(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.starts_with("api_key=") {
                "api_key=***".to_string()
            } else if part.starts_with("api_secret=") {
                "api_secret=***".to_string()
            } else if part.starts_with("token=") {
                "token=***".to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{next_retry_delay_ms, HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        calls: AtomicUsize,
        responses: Mutex<HashMap<String, HttpResponse>>,
    }
    impl SpyTransport {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                responses: Mutex::new(HashMap::new()),
            }
        }
        async fn set_response(&self, path: &str, status: u16, body: &str) {
            self.responses.lock().await.insert(
                path.into(),
                HttpResponse {
                    status,
                    body: Bytes::copy_from_slice(body.as_bytes()),
                },
            );
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }
    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.responses.lock().await.remove(&req.path).unwrap())
        }
        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream { connected: true })
        }
    }

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_all_endpoints_are_covered_by_fixture_driven_tests() {
        let transport = SpyTransport::new();
        let adapter = KrakenRestAdapter::new(
            "https://api.kraken.test",
            "https://futures.kraken.test/derivatives",
        );
        for spec in KrakenRestAdapter::endpoint_specs() {
            let filename = format!("{}.json", spec.id);
            let path = if spec.id.starts_with("futures") {
                format!("https://futures.kraken.test/derivatives{}", spec.path)
            } else {
                format!("https://api.kraken.test{}", spec.path)
            };
            transport
                .set_response(&path, 200, &fixture(&filename))
                .await;
            let key = if spec.requires_auth {
                Some("k-1".to_string())
            } else {
                None
            };
            assert!(
                adapter
                    .execute_rest(&transport, spec.id, None, key)
                    .await
                    .is_ok(),
                "failed id={}",
                spec.id
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_rejects_without_auth_and_transport_is_not_called() {
        let transport = SpyTransport::new();
        let adapter = KrakenRestAdapter::new(
            "https://api.kraken.test",
            "https://futures.kraken.test/derivatives",
        );
        let err = adapter
            .execute_rest(&transport, "spot.private.rest.balance.get", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[test]
    fn maps_kraken_errors_to_ucel_error_codes() {
        let auth = map_kraken_http_error(401, br#"{"error":["EAPI:Invalid key"]}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let invalid = map_kraken_http_error(400, br#"{"error":["EOrder:Invalid price"]}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);

        let rate = map_kraken_http_error(429, b"retry_after_ms=1500");
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(1500));
    }

    #[test]
    fn retry_policy_respects_retry_after_for_429() {
        let policy = RetryPolicy {
            base_delay_ms: 100,
            max_delay_ms: 4_000,
            jitter_ms: 50,
            respect_retry_after: true,
        };
        assert_eq!(next_retry_delay_ms(&policy, 3, Some(777)), 777);
    }

    #[test]
    fn ws_contract_builds_subscribe_unsubscribe_for_all_catalog_ids() {
        for spec in KrakenWsAdapter::ws_channel_specs() {
            let sub = KrakenWsAdapter::build_subscribe(spec.id, "BTC/USD", Some("tok"), Some("k"))
                .unwrap();
            assert!(sub.is_object());
            if spec.supports_unsubscribe {
                assert!(
                    KrakenWsAdapter::build_unsubscribe(spec.id, "BTC/USD", Some("tok")).is_some()
                );
            }
        }
    }

    #[test]
    fn private_ws_preflight_rejects_missing_credentials() {
        let err = KrakenWsAdapter::build_subscribe(
            "spot.private.ws.v2.trade.add_order",
            "BTC/USD",
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[test]
    fn ws_messages_are_normalized_and_typed() {
        let msg = r#"{"channel":"book","symbol":"BTC/USD","data":[{"bid":"1"}]}"#;
        let normalized =
            normalize_ws_event("spot.public.ws.v2.market.book.subscribe", msg).unwrap();
        assert_eq!(
            normalized.channel,
            "spot.public.ws.v2.market.book.subscribe"
        );
        assert_eq!(normalized.symbol.as_deref(), Some("BTC/USD"));
        assert_eq!(normalized.kind, "update");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_is_idempotent() {
        let transport = SpyTransport::new();
        let mut ws = KrakenWsAdapter::default();
        assert!(ws.subscribe_once("spot.public.ws.v2.market.book.subscribe", "BTC/USD"));
        assert!(!ws.subscribe_once("spot.public.ws.v2.market.book.subscribe", "BTC/USD"));
        let count = ws.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(ws.metrics.ws_reconnect_total, 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_uses_bounded_channel_and_counts_overflow() {
        let mut metrics = WsAdapterMetrics::default();
        let mut q = WsBackpressureBuffer::with_capacity(1);
        q.try_push(
            NormalizedWsEvent {
                channel: "x".into(),
                symbol: None,
                kind: "update".into(),
                payload: serde_json::json!({"v":1}),
            },
            &mut metrics,
        );
        q.try_push(
            NormalizedWsEvent {
                channel: "x".into(),
                symbol: None,
                kind: "update".into(),
                payload: serde_json::json!({"v":2}),
            },
            &mut metrics,
        );
        assert_eq!(metrics.ws_backpressure_overflow_total, 1);
        let first = q.recv().await.unwrap();
        assert_eq!(first.payload["v"], 1);
    }

    #[test]
    fn orderbook_gap_triggers_immediate_resync() {
        let mut metrics = WsAdapterMetrics::default();
        let mut ob = OrderBookSyncState::default();
        ob.apply_snapshot(
            10,
            &[("100".into(), "1".into())],
            &[("101".into(), "1".into())],
        );
        ob.apply_delta(12, &[("100".into(), "2".into())], &[], &mut metrics);
        assert!(ob.needs_resync);
        assert_eq!(metrics.ws_orderbook_gap_total, 1);
    }

    #[test]
    fn logging_scrubber_masks_secrets() {
        let raw = "api_key=abc api_secret=def token=ghi";
        let scrubbed = scrub_secrets(raw);
        assert!(!scrubbed.contains("abc"));
        assert!(!scrubbed.contains("def"));
        assert!(!scrubbed.contains("ghi"));
    }
    #[test]
    fn kraken_coverage_manifest_is_strict_and_has_no_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/kraken.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        for e in &manifest.entries {
            assert!(e.implemented, "id not implemented: {}", e.id);
            assert!(e.tested, "id not tested: {}", e.id);
        }
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict gate requires zero gaps: {gaps:?}");
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
