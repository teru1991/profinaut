use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use ucel_core::{ErrorCode, Exchange, OpName, UcelError};
use ucel_core::{OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent};
use ucel_transport::{
    enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport, WsConnectRequest,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 7] = [
    EndpointSpec {
        id: "coinm.public.rest.general.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/general-info",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.errors.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/error-code",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.common.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/common-definition",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.market.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/market-data/rest-api",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.private.rest.trade.ref",
        method: "POST/GET/DELETE",
        base_url: "docs://binance-coinm",
        path: "/trade/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "coinm.private.rest.account.ref",
        method: "GET/POST",
        base_url: "docs://binance-coinm",
        path: "/account/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "coinm.private.rest.listenkey.ref",
        method: "POST/PUT/DELETE",
        base_url: "https://dapi.binance.com",
        path: "/dapi/v1/listenKey",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub enum BinanceCoinmRestResponse {
    General(RefPageResponse),
    Errors(RefPageResponse),
    Common(RefPageResponse),
    Market(RefPageResponse),
    Trade(RefPageResponse),
    Account(RefPageResponse),
    ListenKey(ListenKeyResponse),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RefPageResponse {
    pub section: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

#[derive(Clone)]
pub struct BinanceCoinmRestAdapter {
    docs_base_url: Arc<str>,
    api_base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BinanceCoinmRestAdapter {
    pub fn new(docs_base_url: impl Into<String>, api_base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            docs_base_url: Arc::from(docs_base_url.into()),
            api_base_url: Arc::from(api_base_url.into()),
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
    ) -> Result<BinanceCoinmRestResponse, UcelError> {
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
            venue: "binance-coinm".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let base = if spec.base_url == "https://dapi.binance.com" {
            self.api_base_url.as_ref()
        } else {
            self.docs_base_url.as_ref()
        };

        let req = HttpRequest {
            method: spec.method.into(),
            path: format!("{base}{}", spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_binance_coinm_http_error(
                response.status,
                &response.body,
            ));
        }

        let parsed = match endpoint_id {
            "coinm.public.rest.general.ref" => {
                BinanceCoinmRestResponse::General(parse_json(&response.body)?)
            }
            "coinm.public.rest.errors.ref" => {
                BinanceCoinmRestResponse::Errors(parse_json(&response.body)?)
            }
            "coinm.public.rest.common.ref" => {
                BinanceCoinmRestResponse::Common(parse_json(&response.body)?)
            }
            "coinm.public.rest.market.ref" => {
                BinanceCoinmRestResponse::Market(parse_json(&response.body)?)
            }
            "coinm.private.rest.trade.ref" => {
                BinanceCoinmRestResponse::Trade(parse_json(&response.body)?)
            }
            "coinm.private.rest.account.ref" => {
                BinanceCoinmRestResponse::Account(parse_json(&response.body)?)
            }
            "coinm.private.rest.listenkey.ref" => {
                BinanceCoinmRestResponse::ListenKey(parse_json(&response.body)?)
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

#[derive(Debug, Deserialize)]
struct BinanceCoinmErrorEnvelope {
    code: Option<i64>,
}

pub fn map_binance_coinm_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|b| b.split("retry_after_ms=").nth(1))
            .and_then(|v| v.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let code = serde_json::from_slice::<BinanceCoinmErrorEnvelope>(body)
        .ok()
        .and_then(|v| v.code)
        .unwrap_or_default();

    let mut err = match code {
        -2015 | -2014 | -1022 => UcelError::new(ErrorCode::AuthFailed, "authentication failed"),
        -2010 | -2011 | -1116 | -1111 => UcelError::new(ErrorCode::InvalidOrder, "invalid order"),
        -1003 | -1015 => UcelError::new(ErrorCode::RateLimited, "rate limited"),
        -1002 | -2017 => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("binance-coinm http error status={status}"),
        ),
    };
    err.key_specific = matches!(
        err.code,
        ErrorCode::AuthFailed | ErrorCode::PermissionDenied
    );
    err
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

impl Exchange for BinanceCoinmRestAdapter {
    fn name(&self) -> &'static str {
        "binance-coinm"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("op {} not implemented", op),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub requires_auth: bool,
}

const WS_CHANNELS: [WsChannelSpec; 18] = [
    WsChannelSpec {
        id: "coinm.public.ws.market.root",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.aggtrade",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.markprice",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.kline",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.continuous-kline",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.index-kline",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.miniticker",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.miniticker.all",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.ticker",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.ticker.all",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.bookticker",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.liquidation",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.depth.partial",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.depth.diff",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.composite-index",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.market.contract-info",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.public.ws.wsapi.general",
        ws_url: "wss://ws-dapi.binance.com/ws-dapi/v1",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "coinm.private.ws.userdata.events",
        ws_url: "wss://dstream.binance.com/ws",
        requires_auth: true,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub symbol: Option<String>,
    pub listen_key: Option<String>,
    pub key_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
    pub ws_orderbook_resync_total: AtomicU64,
}

pub struct BinanceCoinmBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}

impl BinanceCoinmBackpressure {
    pub fn new(cap: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx, counters }
    }

    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.counters
                .ws_backpressure_drops_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderBookHealth {
    Recovered,
    Degraded,
}

#[derive(Debug, Default)]
pub struct OrderBookResyncEngine {
    buffered_deltas: VecDeque<OrderBookDelta>,
    expected_next: Option<u64>,
    health: OrderBookHealth,
}

impl Default for OrderBookHealth {
    fn default() -> Self {
        Self::Recovered
    }
}

impl OrderBookResyncEngine {
    pub fn ingest_delta(&mut self, delta: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(next) = self.expected_next {
            if delta.sequence_start != next {
                self.expected_next = None;
                self.buffered_deltas.clear();
                self.health = OrderBookHealth::Degraded;
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "orderbook gap detected; immediate resync required",
                ));
            }
            self.expected_next = Some(delta.sequence_end + 1);
        } else {
            self.buffered_deltas.push_back(delta);
        }
        Ok(())
    }

    pub fn apply_snapshot(
        &mut self,
        mut snapshot: OrderBookSnapshot,
    ) -> Result<OrderBookSnapshot, UcelError> {
        self.expected_next = Some(snapshot.sequence + 1);
        while let Some(delta) = self.buffered_deltas.pop_front() {
            if delta.sequence_end <= snapshot.sequence {
                continue;
            }
            if delta.sequence_start > snapshot.sequence + 1 {
                self.health = OrderBookHealth::Degraded;
                return Err(UcelError::new(ErrorCode::Desync, "snapshot mismatch"));
            }
            merge_levels(&mut snapshot.bids, delta.bids, true);
            merge_levels(&mut snapshot.asks, delta.asks, false);
            snapshot.sequence = delta.sequence_end;
        }
        self.health = OrderBookHealth::Recovered;
        self.expected_next = Some(snapshot.sequence + 1);
        Ok(snapshot)
    }

    pub fn health(&self) -> OrderBookHealth {
        self.health
    }
}

fn merge_levels(target: &mut Vec<OrderBookLevel>, updates: Vec<OrderBookLevel>, desc: bool) {
    for update in updates {
        if let Some(existing) = target.iter_mut().find(|x| x.price == update.price) {
            existing.qty = update.qty;
        } else {
            target.push(update);
        }
    }
    target.retain(|x| x.qty > 0.0);
    target.sort_by(|a, b| {
        if desc {
            b.price.partial_cmp(&a.price)
        } else {
            a.price.partial_cmp(&b.price)
        }
        .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[derive(Debug, Clone)]
pub struct WsCommand {
    pub method: &'static str,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Trade(TradeEvent),
    Generic {
        event: String,
        symbol: Option<String>,
    },
    OrderBookDelta(OrderBookDelta),
    WsApiResponse {
        status: u16,
    },
    UserData {
        event: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct GenericWsMsg {
    e: String,
    #[serde(rename = "s")]
    symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TradeWsMsg {
    e: String,
    s: String,
    p: String,
    q: String,
    m: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct DepthWsMsg {
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "u")]
    final_update_id: u64,
    b: Vec<(String, String)>,
    a: Vec<(String, String)>,
}

#[derive(Debug, Clone, Deserialize)]
struct WsApiMsg {
    status: u16,
}

#[derive(Debug, Clone, Deserialize)]
struct UserDataMsg {
    e: String,
}

#[derive(Clone)]
pub struct BinanceCoinmWsAdapter {
    subscriptions: Arc<Mutex<HashSet<WsSubscription>>>,
    counters: Arc<WsCounters>,
}

impl BinanceCoinmWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            counters,
        }
    }

    pub fn channel_specs() -> &'static [WsChannelSpec] {
        &WS_CHANNELS
    }

    pub fn counters(&self) -> Arc<WsCounters> {
        self.counters.clone()
    }

    pub async fn subscribe<T: Transport>(
        &self,
        transport: &T,
        subscription: WsSubscription,
    ) -> Result<bool, UcelError> {
        let spec = WS_CHANNELS
            .iter()
            .find(|s| s.id == subscription.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeTrades,
            venue: "binance-coinm".into(),
            policy_id: "default".into(),
            key_id: subscription.key_id.clone(),
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let ws_url = resolve_ws_url(spec, &subscription)?;
        transport
            .connect_ws(WsConnectRequest { url: ws_url }, ctx)
            .await?;

        let mut guard = self.subscriptions.lock().await;
        Ok(guard.insert(subscription))
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(
        &self,
        transport: &T,
    ) -> Result<usize, UcelError> {
        let subs: Vec<WsSubscription> = self.subscriptions.lock().await.iter().cloned().collect();
        self.counters
            .ws_reconnect_total
            .fetch_add(1, Ordering::Relaxed);
        for sub in &subs {
            let spec = WS_CHANNELS
                .iter()
                .find(|s| s.id == sub.channel_id)
                .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
            let ctx = RequestContext {
                trace_id: Uuid::new_v4().to_string(),
                request_id: Uuid::new_v4().to_string(),
                run_id: Uuid::new_v4().to_string(),
                op: OpName::SubscribeTrades,
                venue: "binance-coinm".into(),
                policy_id: "default".into(),
                key_id: sub.key_id.clone(),
                requires_auth: spec.requires_auth,
            };
            transport
                .connect_ws(
                    WsConnectRequest {
                        url: resolve_ws_url(spec, sub)?,
                    },
                    ctx,
                )
                .await?;
        }
        self.counters
            .ws_resubscribe_total
            .fetch_add(subs.len() as u64, Ordering::Relaxed);
        Ok(subs.len())
    }

    pub fn build_subscribe_command(&self, sub: &WsSubscription) -> WsCommand {
        WsCommand {
            method: "SUBSCRIBE",
            params: vec![stream_name(sub)],
        }
    }

    pub fn build_unsubscribe_command(&self, sub: &WsSubscription) -> WsCommand {
        WsCommand {
            method: "UNSUBSCRIBE",
            params: vec![stream_name(sub)],
        }
    }

    pub fn parse_market_event(
        &self,
        channel_id: &str,
        body: &Bytes,
    ) -> Result<MarketEvent, UcelError> {
        match channel_id {
            "coinm.public.ws.market.aggtrade" => {
                let msg: TradeWsMsg = parse_json(body)?;
                Ok(MarketEvent::Trade(TradeEvent {
                    trade_id: format!("{}:{}", msg.s, msg.p),
                    price: parse_num(&msg.p)?,
                    qty: parse_num(&msg.q)?,
                    side: if msg.m { "sell".into() } else { "buy".into() },
                }))
            }
            "coinm.public.ws.market.depth.partial" | "coinm.public.ws.market.depth.diff" => {
                let msg: DepthWsMsg = parse_json(body)?;
                let bids = msg
                    .b
                    .into_iter()
                    .map(|(p, q)| {
                        Ok(OrderBookLevel {
                            price: parse_num(&p)?,
                            qty: parse_num(&q)?,
                        })
                    })
                    .collect::<Result<Vec<_>, UcelError>>()?;
                let asks = msg
                    .a
                    .into_iter()
                    .map(|(p, q)| {
                        Ok(OrderBookLevel {
                            price: parse_num(&p)?,
                            qty: parse_num(&q)?,
                        })
                    })
                    .collect::<Result<Vec<_>, UcelError>>()?;
                Ok(MarketEvent::OrderBookDelta(OrderBookDelta {
                    bids,
                    asks,
                    sequence_start: msg.first_update_id,
                    sequence_end: msg.final_update_id,
                }))
            }
            "coinm.public.ws.wsapi.general" => {
                let msg: WsApiMsg = parse_json(body)?;
                Ok(MarketEvent::WsApiResponse { status: msg.status })
            }
            "coinm.private.ws.userdata.events" => {
                let msg: UserDataMsg = parse_json(body)?;
                Ok(MarketEvent::UserData { event: msg.e })
            }
            _ => {
                let msg: GenericWsMsg = parse_json(body)?;
                Ok(MarketEvent::Generic {
                    event: msg.e,
                    symbol: msg.symbol,
                })
            }
        }
    }
}

fn resolve_ws_url(spec: &WsChannelSpec, sub: &WsSubscription) -> Result<String, UcelError> {
    if spec.id == "coinm.private.ws.userdata.events" {
        let listen_key = sub
            .listen_key
            .clone()
            .ok_or_else(|| UcelError::new(ErrorCode::MissingAuth, "listen key required"))?;
        return Ok(format!("{}/{}", spec.ws_url, listen_key));
    }
    Ok(spec.ws_url.to_string())
}

fn stream_name(sub: &WsSubscription) -> String {
    if sub.channel_id == "coinm.private.ws.userdata.events" {
        return "userdata".into();
    }
    match &sub.symbol {
        Some(symbol) => format!("{}:{}", sub.channel_id, symbol),
        None => sub.channel_id.clone(),
    }
}

fn parse_num(raw: &str) -> Result<f64, UcelError> {
    raw.parse::<f64>()
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("parse number failed: {e}")))
}

pub fn log_private_ws_auth_attempt(key_id: Option<&str>, _api_key: &str, _api_secret: &str) {
    tracing::info!(key_id = ?key_id, "binance coinm private ws auth attempt");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use tracing_subscriber::fmt::MakeWriter;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Debug, Deserialize)]
    struct CoverageManifest {
        venue: String,
        strict: bool,
        entries: Vec<CoverageEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct CoverageEntry {
        id: String,
        implemented: bool,
        tested: bool,
    }

    struct SpyTransport {
        calls: AtomicUsize,
        ws_calls: AtomicUsize,
        key_ids: Mutex<Vec<Option<String>>>,
        responses: Mutex<HashMap<String, HttpResponse>>,
    }

    impl SpyTransport {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                ws_calls: AtomicUsize::new(0),
                key_ids: Mutex::new(Vec::new()),
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

        fn ws_calls(&self) -> usize {
            self.ws_calls.load(Ordering::Relaxed)
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            req: HttpRequest,
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.key_ids.lock().await.push(ctx.key_id.clone());
            self.responses
                .lock()
                .await
                .remove(&req.path)
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing mocked response"))
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_calls.fetch_add(1, Ordering::Relaxed);
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
    async fn rest_contract_all_catalog_endpoints_parse_with_fixtures() {
        let transport = SpyTransport::new();
        let adapter =
            BinanceCoinmRestAdapter::new("https://docs.test/binance-coinm", "https://dapi.test");

        for spec in BinanceCoinmRestAdapter::endpoint_specs() {
            let filename = format!("{}.json", spec.id);
            let base = if spec.base_url == "https://dapi.binance.com" {
                "https://dapi.test"
            } else {
                "https://docs.test/binance-coinm"
            };
            let path = format!("{base}{}", spec.path);
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

        let keys = transport.key_ids.lock().await.clone();
        assert!(
            keys.iter().any(|k| k.is_none()),
            "public route must use no key path"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_auth_and_transport_is_not_called() {
        let transport = SpyTransport::new();
        let adapter =
            BinanceCoinmRestAdapter::new("https://docs.test/binance-coinm", "https://dapi.test");
        let err = adapter
            .execute_rest(&transport, "coinm.private.rest.trade.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ws_contract_all_channels_typed_parse() {
        let ws = BinanceCoinmWsAdapter::new(Arc::new(WsCounters::default()));
        for spec in BinanceCoinmWsAdapter::channel_specs() {
            let fname = format!("{}.json", spec.id);
            let body = Bytes::from(fixture(&fname));
            assert!(
                ws.parse_market_event(spec.id, &body).is_ok(),
                "id={}",
                spec.id
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ws_reconnect_resubscribe_and_idempotent() {
        let transport = SpyTransport::new();
        let counters = Arc::new(WsCounters::default());
        let ws = BinanceCoinmWsAdapter::new(counters.clone());

        let sub = WsSubscription {
            channel_id: "coinm.public.ws.market.aggtrade".into(),
            symbol: Some("BTCUSD_PERP".into()),
            listen_key: None,
            key_id: None,
        };
        assert!(ws.subscribe(&transport, sub.clone()).await.unwrap());
        assert!(!ws.subscribe(&transport, sub).await.unwrap());

        let count = ws.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(counters.ws_reconnect_total.load(Ordering::Relaxed), 1);
        assert_eq!(counters.ws_resubscribe_total.load(Ordering::Relaxed), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ws_private_preflight_rejects_without_key_and_spy_shows_no_connect() {
        let transport = SpyTransport::new();
        let ws = BinanceCoinmWsAdapter::new(Arc::new(WsCounters::default()));
        let sub = WsSubscription {
            channel_id: "coinm.private.ws.userdata.events".into(),
            symbol: None,
            listen_key: Some("lk".into()),
            key_id: None,
        };
        let err = ws.subscribe(&transport, sub).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.ws_calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_bounded_channel_drops_and_metrics() {
        let counters = Arc::new(WsCounters::default());
        let mut bp = BinanceCoinmBackpressure::new(1, counters.clone());
        bp.try_enqueue(Bytes::from_static(b"one"));
        bp.try_enqueue(Bytes::from_static(b"two"));
        assert_eq!(
            counters.ws_backpressure_drops_total.load(Ordering::Relaxed),
            1
        );
        let got = bp.recv().await.unwrap();
        assert_eq!(got, Bytes::from_static(b"one"));
    }

    #[test]
    fn orderbook_gap_triggers_resync_then_recovered() {
        let mut e = OrderBookResyncEngine::default();
        e.ingest_delta(OrderBookDelta {
            bids: vec![OrderBookLevel {
                price: 100.0,
                qty: 1.0,
            }],
            asks: vec![],
            sequence_start: 5,
            sequence_end: 6,
        })
        .unwrap();
        let snapshot = e
            .apply_snapshot(OrderBookSnapshot {
                bids: vec![OrderBookLevel {
                    price: 99.0,
                    qty: 1.0,
                }],
                asks: vec![OrderBookLevel {
                    price: 101.0,
                    qty: 1.0,
                }],
                sequence: 4,
            })
            .unwrap();
        assert_eq!(snapshot.sequence, 6);

        let err = e
            .ingest_delta(OrderBookDelta {
                bids: vec![],
                asks: vec![],
                sequence_start: 9,
                sequence_end: 9,
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert_eq!(e.health(), OrderBookHealth::Degraded);
    }

    #[test]
    fn no_secret_leak_in_tracing_logs() {
        #[derive(Clone, Default)]
        struct SharedBuf(Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for SharedBuf {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        #[derive(Clone)]
        struct WriterMaker(SharedBuf);
        impl<'a> MakeWriter<'a> for WriterMaker {
            type Writer = SharedBuf;
            fn make_writer(&'a self) -> Self::Writer {
                self.0.clone()
            }
        }

        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(WriterMaker(buf.clone()))
            .without_time()
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        log_private_ws_auth_attempt(Some("kid-1"), "api_key_123", "api_secret_456");
        let logs = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
        assert!(logs.contains("kid-1"));
        assert!(!logs.contains("api_key_123"));
        assert!(!logs.contains("api_secret_456"));
    }

    #[test]
    fn endpoint_specs_match_catalog_rest_and_ws_ids_exactly() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let raw =
            std::fs::read_to_string(repo_root.join("docs/exchanges/binance-coinm/catalog.json"))
                .unwrap();
        let catalog: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut impl_ids: Vec<&str> = BinanceCoinmRestAdapter::endpoint_specs()
            .iter()
            .map(|e| e.id)
            .chain(BinanceCoinmWsAdapter::channel_specs().iter().map(|e| e.id))
            .collect();
        let mut catalog_ids: Vec<String> = catalog["rest_endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap().to_string())
            .chain(
                catalog["ws_channels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| e["id"].as_str().unwrap().to_string()),
            )
            .collect();
        impl_ids.sort_unstable();
        catalog_ids.sort_unstable();
        assert_eq!(
            impl_ids,
            catalog_ids.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    #[test]
    fn coverage_manifest_has_no_gaps_and_is_strict() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../coverage/binance-coinm.yaml");
        let raw = std::fs::read_to_string(manifest_path).unwrap();
        let manifest: CoverageManifest = serde_yaml::from_str(&raw).unwrap();
        assert_eq!(manifest.venue, "binance-coinm");
        assert!(manifest.strict);
        for e in &manifest.entries {
            assert!(e.implemented, "id not implemented: {}", e.id);
            assert!(e.tested, "id not tested: {}", e.id);
        }
    }
}

pub mod symbols;
pub mod ws;
pub mod ws_manager;
pub mod channels;
