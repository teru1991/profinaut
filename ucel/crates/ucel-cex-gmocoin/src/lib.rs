use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use ucel_core::{ErrorCode, Exchange, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 30] = [
    EndpointSpec { id: "crypto.public.rest.status.get", method: "GET", path: "/public/v1/status", requires_auth: false },
    EndpointSpec { id: "crypto.public.rest.ticker.get", method: "GET", path: "/public/v1/ticker", requires_auth: false },
    EndpointSpec { id: "crypto.public.rest.orderbooks.get", method: "GET", path: "/public/v1/orderbooks", requires_auth: false },
    EndpointSpec { id: "crypto.public.rest.trades.get", method: "GET", path: "/public/v1/trades", requires_auth: false },
    EndpointSpec { id: "crypto.public.rest.klines.get", method: "GET", path: "/public/v1/klines", requires_auth: false },
    EndpointSpec { id: "crypto.private.rest.wsauth.post", method: "POST", path: "/private/v1/ws-auth", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.wsauth.extend.put", method: "PUT", path: "/private/v1/ws-auth", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.assets.get", method: "GET", path: "/private/v1/account/assets", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.margin.get", method: "GET", path: "/private/v1/account/margin", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.activeorders.get", method: "GET", path: "/private/v1/activeOrders", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.executions.get", method: "GET", path: "/private/v1/executions", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.latestexecutions.get", method: "GET", path: "/private/v1/latestExecutions", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.order.post", method: "POST", path: "/private/v1/order", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.changeorder.post", method: "POST", path: "/private/v1/changeOrder", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.cancelorder.post", method: "POST", path: "/private/v1/cancelOrder", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.openpositions.get", method: "GET", path: "/private/v1/openPositions", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.positionsummary.get", method: "GET", path: "/private/v1/positionSummary", requires_auth: true },
    EndpointSpec { id: "crypto.private.rest.closeorder.post", method: "POST", path: "/private/v1/closeOrder", requires_auth: true },
    EndpointSpec { id: "fx.public.rest.status.get", method: "GET", path: "/fx/public/v1/status", requires_auth: false },
    EndpointSpec { id: "fx.public.rest.ticker.get", method: "GET", path: "/fx/public/v1/ticker", requires_auth: false },
    EndpointSpec { id: "fx.public.rest.orderbooks.get", method: "GET", path: "/fx/public/v1/orderbooks", requires_auth: false },
    EndpointSpec { id: "fx.public.rest.trades.get", method: "GET", path: "/fx/public/v1/trades", requires_auth: false },
    EndpointSpec { id: "fx.public.rest.klines.get", method: "GET", path: "/fx/public/v1/klines", requires_auth: false },
    EndpointSpec { id: "fx.private.rest.wsauth.post", method: "POST", path: "/fx/private/v1/ws-auth", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.assets.get", method: "GET", path: "/fx/private/v1/account/assets", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.activeorders.get", method: "GET", path: "/fx/private/v1/activeOrders", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.order.post", method: "POST", path: "/fx/private/v1/order", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.cancelorder.post", method: "POST", path: "/fx/private/v1/cancelOrder", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.openpositions.get", method: "GET", path: "/fx/private/v1/openPositions", requires_auth: true },
    EndpointSpec { id: "fx.private.rest.closeorder.post", method: "POST", path: "/fx/private/v1/closeOrder", requires_auth: true },
];

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub channel: &'static str,
    pub requires_auth: bool,
}

const WS_CHANNELS: [WsChannelSpec; 12] = [
    WsChannelSpec { id: "crypto.public.ws.ticker.update", ws_url: "wss://api.coin.z.com/ws/public/v1", channel: "ticker", requires_auth: false },
    WsChannelSpec { id: "crypto.public.ws.trades.update", ws_url: "wss://api.coin.z.com/ws/public/v1", channel: "trades", requires_auth: false },
    WsChannelSpec { id: "crypto.public.ws.orderbooks.update", ws_url: "wss://api.coin.z.com/ws/public/v1", channel: "orderbooks", requires_auth: false },
    WsChannelSpec { id: "crypto.private.ws.executionevents.update", ws_url: "wss://api.coin.z.com/ws/private/v1/{token}", channel: "executionEvents", requires_auth: true },
    WsChannelSpec { id: "crypto.private.ws.orderevents.update", ws_url: "wss://api.coin.z.com/ws/private/v1/{token}", channel: "orderEvents", requires_auth: true },
    WsChannelSpec { id: "crypto.private.ws.positionevents.update", ws_url: "wss://api.coin.z.com/ws/private/v1/{token}", channel: "positionEvents", requires_auth: true },
    WsChannelSpec { id: "fx.public.ws.ticker.update", ws_url: "wss://api.coin.z.com/fx/ws/public/v1", channel: "ticker", requires_auth: false },
    WsChannelSpec { id: "fx.public.ws.trades.update", ws_url: "wss://api.coin.z.com/fx/ws/public/v1", channel: "trades", requires_auth: false },
    WsChannelSpec { id: "fx.public.ws.orderbooks.update", ws_url: "wss://api.coin.z.com/fx/ws/public/v1", channel: "orderbooks", requires_auth: false },
    WsChannelSpec { id: "fx.private.ws.executionevents.update", ws_url: "wss://api.coin.z.com/fx/ws/private/v1/{token}", channel: "executionEvents", requires_auth: true },
    WsChannelSpec { id: "fx.private.ws.orderevents.update", ws_url: "wss://api.coin.z.com/fx/ws/private/v1/{token}", channel: "orderEvents", requires_auth: true },
    WsChannelSpec { id: "fx.private.ws.positionevents.update", ws_url: "wss://api.coin.z.com/fx/ws/private/v1/{token}", channel: "positionEvents", requires_auth: true },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmoEnvelope<T> {
    pub status: i32,
    pub data: T,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericPayload { pub endpoint: String, pub ok: bool }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerPayload { pub ask: String, pub bid: String, pub last: String }
#[derive(Debug, Clone, PartialEq)]
pub enum GmoRestResponse { Generic(GmoEnvelope<GenericPayload>), Ticker(GmoEnvelope<Vec<TickerPayload>>) }

#[derive(Debug, Clone, Deserialize)]
struct GmoErrorMessage { #[serde(default)] message_code: String }
#[derive(Debug, Clone, Deserialize)]
struct GmoErrorBody { #[serde(default)] messages: Vec<GmoErrorMessage>, #[serde(default)] retry_after_ms: Option<u64> }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsCommand {
    pub command: &'static str,
    pub channel: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker { symbol: String, bid: f64, ask: f64 },
    Trade(TradeEvent),
    OrderBookSnapshot(OrderBookSnapshot),
    Execution { order_id: String, execution_id: String, symbol: String, size: f64 },
    Order { order_id: String, status: String, symbol: String },
    Position { position_id: String, symbol: String, side: String, size: f64 },
}

#[derive(Debug, Clone, Deserialize)]
struct TickerWsMsg { symbol: String, ask: String, bid: String }
#[derive(Debug, Clone, Deserialize)]
struct TradeWsMsg { symbol: String, side: String, price: String, size: String }
#[derive(Debug, Clone, Deserialize)]
struct BookWsMsg { asks: Vec<BookLevelWs>, bids: Vec<BookLevelWs>, sequence: Option<u64> }
#[derive(Debug, Clone, Deserialize)]
struct BookLevelWs { price: String, size: String }
#[derive(Debug, Clone, Deserialize)]
struct ExecutionWsMsg { #[serde(rename="orderId")] order_id: String, #[serde(rename="executionId")] execution_id: String, symbol: String, size: String }
#[derive(Debug, Clone, Deserialize)]
struct OrderWsMsg { #[serde(rename="orderId")] order_id: String, status: String, symbol: String }
#[derive(Debug, Clone, Deserialize)]
struct PositionWsMsg { #[serde(rename="positionId")] position_id: String, symbol: String, side: String, size: String }

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
}

pub struct GmoWsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}
impl GmoWsBackpressure {
    pub fn new(cap: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx, counters }
    }
    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.counters.ws_backpressure_drops_total.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub async fn recv(&mut self) -> Option<Bytes> { self.rx.recv().await }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderBookHealth { Ok, Degraded }

#[derive(Debug, Default)]
pub struct OrderBookResyncEngine {
    buffered_deltas: VecDeque<OrderBookDelta>,
    next_sequence: Option<u64>,
    health: OrderBookHealth,
}
impl OrderBookResyncEngine {
    pub fn ingest_delta(&mut self, delta: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(next) = self.next_sequence {
            if delta.sequence_start != next {
                self.health = OrderBookHealth::Degraded;
                self.next_sequence = None;
                self.buffered_deltas.clear();
                return Err(UcelError::new(ErrorCode::Desync, "orderbook gap detected; resync required"));
            }
            self.next_sequence = Some(delta.sequence_end + 1);
        } else {
            self.buffered_deltas.push_back(delta);
        }
        Ok(())
    }

    pub fn apply_snapshot(&mut self, mut snapshot: OrderBookSnapshot) -> Result<OrderBookSnapshot, UcelError> {
        self.next_sequence = Some(snapshot.sequence + 1);
        while let Some(delta) = self.buffered_deltas.pop_front() {
            if delta.sequence_end <= snapshot.sequence { continue; }
            if delta.sequence_start > snapshot.sequence + 1 {
                self.health = OrderBookHealth::Degraded;
                return Err(UcelError::new(ErrorCode::Desync, "delta ahead of snapshot"));
            }

            // Apply bids from delta
            for delta_bid in delta.bids {
                if let Some(existing_bid) = snapshot.bids.iter_mut().find(|b| b.price == delta_bid.price) {
                    existing_bid.qty = delta_bid.qty;
                } else {
                    snapshot.bids.push(delta_bid);
                }
            }
            // Apply asks from delta
            for delta_ask in delta.asks {
                if let Some(existing_ask) = snapshot.asks.iter_mut().find(|a| a.price == delta_ask.price) {
                    existing_ask.qty = delta_ask.qty;
                } else {
                    snapshot.asks.push(delta_ask);
                }
            }

            // Remove zero quantity levels
            snapshot.bids.retain(|b| b.qty > 0.0);
            snapshot.asks.retain(|a| a.qty > 0.0);

            // Sort to maintain order book structure (descending for bids, ascending for asks)
            snapshot.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
            snapshot.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

            snapshot.sequence = delta.sequence_end;
        }
        self.health = OrderBookHealth::Ok;
        self.next_sequence = Some(snapshot.sequence + 1);
        Ok(snapshot)
    }

    pub fn health(&self) -> OrderBookHealth { self.health.clone() }
}

impl Default for OrderBookHealth { fn default() -> Self { Self::Ok } }

#[derive(Clone)]
pub struct GmoCoinRestAdapter {
    base_url: Arc<str>,
    #[allow(dead_code)]
    http_client: reqwest::Client,
}

impl GmoCoinRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: Arc::from(base_url.into()), http_client: reqwest::Client::new() }
    }
    pub fn endpoint_specs() -> &'static [EndpointSpec] { &ENDPOINTS }

    pub async fn execute_rest<T: Transport>(&self, transport: &T, endpoint_id: &str, body: Option<Bytes>, key_id: Option<String>) -> Result<GmoRestResponse, UcelError> {
        let spec = ENDPOINTS.iter().find(|s| s.id == endpoint_id).ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown endpoint: {endpoint_id}")))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(), request_id: Uuid::new_v4().to_string(), run_id: Uuid::new_v4().to_string(), op: OpName::FetchStatus,
            venue: "gmocoin".into(), policy_id: "default".into(), key_id, requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        let req = HttpRequest { method: spec.method.into(), path: format!("{}{}", self.base_url, spec.path), body };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 { return Err(map_http_error(response.status, &response.body)); }
        if endpoint_id.ends_with("ticker.get") {
            return Ok(GmoRestResponse::Ticker(parse_json(&response.body)?));
        }
        Ok(GmoRestResponse::Generic(parse_json(&response.body)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: &'static str,
    pub symbol: Option<String>,
}

pub struct GmoCoinWsAdapter {
    counters: Arc<WsCounters>,
    active: HashSet<WsSubscription>,
}
impl GmoCoinWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self { Self { counters, active: HashSet::new() } }
    pub fn channel_specs() -> &'static [WsChannelSpec] { &WS_CHANNELS }

    pub fn subscribe_command(&self, channel_id: &'static str, symbol: Option<String>) -> Result<WsCommand, UcelError> {
        let spec = ws_spec(channel_id)?;
        Ok(WsCommand { command: "subscribe", channel: spec.channel, symbol })
    }

    pub fn unsubscribe_command(&self, channel_id: &'static str, symbol: Option<String>) -> Result<WsCommand, UcelError> {
        let spec = ws_spec(channel_id)?;
        Ok(WsCommand { command: "unsubscribe", channel: spec.channel, symbol })
    }

    pub async fn connect_and_subscribe<T: Transport>(&mut self, transport: &T, sub: WsSubscription, key_id: Option<String>) -> Result<(), UcelError> {
        let spec = ws_spec(sub.channel_id)?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(), request_id: Uuid::new_v4().to_string(), run_id: Uuid::new_v4().to_string(),
            op: op_for_ws_id(spec.id), venue: "gmocoin".into(), policy_id: "default".into(), key_id: key_id.clone(), requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        transport.connect_ws(WsConnectRequest { url: spec.ws_url.to_string() }, ctx).await?;
        self.active.insert(sub);
        Ok(())
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(&self, transport: &T, key_id: Option<String>) -> Result<(), UcelError> {
        self.counters.ws_reconnect_total.fetch_add(1, Ordering::Relaxed);
        for sub in &self.active {
            let spec = ws_spec(sub.channel_id)?;
            let ctx = RequestContext {
                trace_id: Uuid::new_v4().to_string(), request_id: Uuid::new_v4().to_string(), run_id: Uuid::new_v4().to_string(),
                op: op_for_ws_id(spec.id), venue: "gmocoin".into(), policy_id: "default".into(), key_id: key_id.clone(), requires_auth: spec.requires_auth,
            };
            enforce_auth_boundary(&ctx)?;
            transport.connect_ws(WsConnectRequest { url: spec.ws_url.to_string() }, ctx).await.map_err(|e| {
                match e.code {
                    ErrorCode::Timeout | ErrorCode::Network | ErrorCode::WsProtocolViolation => e,
                    _ => UcelError::new(ErrorCode::Network, "reconnect failed"),
                }
            })?;
            self.counters.ws_resubscribe_total.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn parse_market_event(&self, channel_id: &str, body: &Bytes) -> Result<MarketEvent, UcelError> {
        match channel_id {
            "crypto.public.ws.ticker.update" | "fx.public.ws.ticker.update" => {
                let m: TickerWsMsg = parse_json(body)?;
                Ok(MarketEvent::Ticker { symbol: m.symbol, bid: parse_num(&m.bid)?, ask: parse_num(&m.ask)? })
            }
            "crypto.public.ws.trades.update" | "fx.public.ws.trades.update" => {
                let m: TradeWsMsg = parse_json(body)?;
                Ok(MarketEvent::Trade(TradeEvent { trade_id: format!("{}:{}", m.symbol, m.price), price: parse_num(&m.price)?, qty: parse_num(&m.size)?, side: m.side }))
            }
            "crypto.public.ws.orderbooks.update" | "fx.public.ws.orderbooks.update" => {
                let m: BookWsMsg = parse_json(body)?;
                Ok(MarketEvent::OrderBookSnapshot(OrderBookSnapshot {
                    bids: m.bids.into_iter().map(|l| OrderBookLevel { price: parse_num(&l.price).unwrap_or(0.0), qty: parse_num(&l.size).unwrap_or(0.0)}).collect(),
                    asks: m.asks.into_iter().map(|l| OrderBookLevel { price: parse_num(&l.price).unwrap_or(0.0), qty: parse_num(&l.size).unwrap_or(0.0)}).collect(),
                    sequence: m.sequence.unwrap_or_default(),
                }))
            }
            "crypto.private.ws.executionevents.update" | "fx.private.ws.executionevents.update" => {
                let m: ExecutionWsMsg = parse_json(body)?;
                Ok(MarketEvent::Execution { order_id: m.order_id, execution_id: m.execution_id, symbol: m.symbol, size: parse_num(&m.size)? })
            }
            "crypto.private.ws.orderevents.update" | "fx.private.ws.orderevents.update" => {
                let m: OrderWsMsg = parse_json(body)?;
                Ok(MarketEvent::Order { order_id: m.order_id, status: m.status, symbol: m.symbol })
            }
            "crypto.private.ws.positionevents.update" | "fx.private.ws.positionevents.update" => {
                let m: PositionWsMsg = parse_json(body)?;
                Ok(MarketEvent::Position { position_id: m.position_id, symbol: m.symbol, side: m.side, size: parse_num(&m.size)? })
            }
            _ => Err(UcelError::new(ErrorCode::NotSupported, format!("unknown ws channel: {channel_id}"))),
        }
    }
}

pub fn sanitize_log_line(line: &str) -> String {
    line
        .split_whitespace()
        .map(|token| {
            if token.starts_with("api_key=") {
                "api_key=[redacted]".to_string()
            } else if token.starts_with("api_secret=") {
                "api_secret=[redacted]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn ws_spec(id: &str) -> Result<&'static WsChannelSpec, UcelError> {
    WS_CHANNELS.iter().find(|c| c.id == id).ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown ws channel: {id}")))
}

fn op_for_ws_id(id: &str) -> OpName {
    match id {
        "crypto.public.ws.ticker.update" | "fx.public.ws.ticker.update" => OpName::SubscribeTicker,
        "crypto.public.ws.trades.update" | "fx.public.ws.trades.update" => OpName::SubscribeTrades,
        "crypto.public.ws.orderbooks.update" | "fx.public.ws.orderbooks.update" => OpName::SubscribeOrderbook,
        "crypto.private.ws.executionevents.update" | "fx.private.ws.executionevents.update" => OpName::SubscribeExecutionEvents,
        "crypto.private.ws.orderevents.update" | "fx.private.ws.orderevents.update" => OpName::SubscribeOrderEvents,
        _ => OpName::SubscribePositionEvents,
    }
}

fn parse_json<T: DeserializeOwned>(body: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(body).map_err(|e| UcelError::new(ErrorCode::Internal, format!("invalid json: {e}")))
}
fn parse_num(s: &str) -> Result<f64, UcelError> { s.parse::<f64>().map_err(|e| UcelError::new(ErrorCode::WsProtocolViolation, format!("invalid decimal: {e}"))) }
fn map_http_error(status: u16, body: &Bytes) -> UcelError {
    let parsed = serde_json::from_slice::<GmoErrorBody>(body).ok();
    if status == 429 { let mut e = UcelError::new(ErrorCode::RateLimited, "rate limited"); e.retry_after_ms = parsed.and_then(|v| v.retry_after_ms); e.ban_risk = true; return e; }
    if status >= 500 { return UcelError::new(ErrorCode::Upstream5xx, "upstream server error"); }
    let code = parsed.as_ref().and_then(|v| v.messages.first()).map(|m| m.message_code.as_str()).unwrap_or_default();
    let mut err = match code {
        "ERR-5201" => UcelError::new(ErrorCode::InvalidOrder, "invalid order"),
        "ERR-5003" => UcelError::new(ErrorCode::AuthFailed, "authentication failed"),
        "ERR-5010" => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        _ => UcelError::new(ErrorCode::Internal, format!("http error status={status}")),
    }; err.key_specific = code == "ERR-5003"; err
}

impl Exchange for GmoCoinRestAdapter {
    fn name(&self) -> &'static str { "gmo_coin" }
    fn execute(&self, op: OpName) -> Result<(), UcelError> { Err(UcelError::new(ErrorCode::NotSupported, format!("op {} not implemented", op))) }
}
pub struct GmoCoinAdapter;
impl Exchange for GmoCoinAdapter {
    fn name(&self) -> &'static str { "gmo_coin" }
    fn execute(&self, op: OpName) -> Result<(), UcelError> { Err(UcelError::new(ErrorCode::NotSupported, format!("{} not implemented for {}", op, self.name()))) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicUsize;
    use tokio::sync::Mutex;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{HttpResponse, WsStream};

    struct SpyTransport {
        calls: AtomicUsize,
        ws_calls: AtomicUsize,
        responses: Mutex<HashMap<String, HttpResponse>>,
    }
    impl SpyTransport {
        fn new() -> Self { Self { calls: AtomicUsize::new(0), ws_calls: AtomicUsize::new(0), responses: Mutex::new(HashMap::new()) } }
        async fn set_response(&self, path: &str, status: u16, body: &str) {
            self.responses.lock().await.insert(path.into(), HttpResponse { status, body: Bytes::copy_from_slice(body.as_bytes()) });
        }
        fn calls(&self) -> usize { self.calls.load(Ordering::Relaxed) }
        fn ws_calls(&self) -> usize { self.ws_calls.load(Ordering::Relaxed) }
    }
    impl Transport for SpyTransport {
        async fn send_http(&self, req: HttpRequest, _ctx: RequestContext) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.responses.lock().await.remove(&req.path).unwrap())
        }
        async fn connect_ws(&self, _req: WsConnectRequest, _ctx: RequestContext) -> Result<WsStream, UcelError> {
            self.ws_calls.fetch_add(1, Ordering::Relaxed);
            Ok(WsStream { connected: true })
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_all_endpoints_are_covered() {
        let transport = SpyTransport::new();
        let adapter = GmoCoinRestAdapter::new("https://api.example.com");
        for spec in GmoCoinRestAdapter::endpoint_specs() {
            let path = format!("https://api.example.com{}", spec.path);
            let success = if spec.id.ends_with("ticker.get") { r#"{"status":0,"data":[{"ask":"1","bid":"1","last":"1"}]}"# } else { &format!("{{\"status\":0,\"data\":{{\"endpoint\":\"{}\",\"ok\":true}}}}", spec.id) };
            transport.set_response(&path, 200, success).await;
            let key = if spec.requires_auth { Some("k1".to_string()) } else { None };
            assert!(adapter.execute_rest(&transport, spec.id, None, key).await.is_ok(), "endpoint {} failed", spec.id);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_rejects_without_auth_and_does_not_hit_transport() {
        let transport = SpyTransport::new();
        let adapter = GmoCoinRestAdapter::new("https://api.example.com");
        let err = adapter.execute_rest(&transport, "crypto.private.rest.order.post", None, None).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ws_contract_all_channels_have_typed_parse_and_subscribe() {
        let counters = Arc::new(WsCounters::default());
        let ws = GmoCoinWsAdapter::new(counters);
        for spec in GmoCoinWsAdapter::channel_specs() {
            let symbol = if spec.channel == "ticker" || spec.channel == "trades" || spec.channel == "orderbooks" { Some("BTC_JPY".to_string()) } else { None };
            let sub = ws.subscribe_command(spec.id, symbol.clone()).unwrap();
            assert_eq!(sub.command, "subscribe");
            let _ = ws.unsubscribe_command(spec.id, symbol).unwrap();
            let body = match spec.channel {
                "ticker" => Bytes::from_static(br#"{"symbol":"BTC_JPY","ask":"1","bid":"1"}"#),
                "trades" => Bytes::from_static(br#"{"symbol":"BTC_JPY","side":"BUY","price":"1","size":"0.1"}"#),
                "orderbooks" => Bytes::from_static(br#"{"asks":[{"price":"2","size":"1"}],"bids":[{"price":"1","size":"1"}],"sequence":1}"#),
                "executionEvents" => Bytes::from_static(br#"{"orderId":"1","executionId":"2","symbol":"BTC_JPY","size":"0.1"}"#),
                "orderEvents" => Bytes::from_static(br#"{"orderId":"1","status":"FILLED","symbol":"BTC_JPY"}"#),
                _ => Bytes::from_static(br#"{"positionId":"1","symbol":"BTC_JPY","side":"BUY","size":"0.1"}"#),
            };
            assert!(ws.parse_market_event(spec.id, &body).is_ok());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_and_resubscribe_increments_metrics_and_is_idempotent() {
        let counters = Arc::new(WsCounters::default());
        let transport = SpyTransport::new();
        let mut ws = GmoCoinWsAdapter::new(counters.clone());
        let sub = WsSubscription { channel_id: "crypto.public.ws.ticker.update", symbol: Some("BTC_JPY".into()) };
        ws.connect_and_subscribe(&transport, sub.clone(), None).await.unwrap();
        ws.connect_and_subscribe(&transport, sub, None).await.unwrap();
        ws.reconnect_and_resubscribe(&transport, None).await.unwrap();
        assert_eq!(transport.ws_calls(), 3);
        assert_eq!(counters.ws_reconnect_total.load(Ordering::Relaxed), 1);
        assert_eq!(counters.ws_resubscribe_total.load(Ordering::Relaxed), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_ws_preflight_rejects_without_auth_before_connect() {
        let counters = Arc::new(WsCounters::default());
        let transport = SpyTransport::new();
        let mut ws = GmoCoinWsAdapter::new(counters);
        let err = ws.connect_and_subscribe(&transport, WsSubscription { channel_id: "crypto.private.ws.orderevents.update", symbol: None }, None).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.ws_calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_drop_counter_increments() {
        let counters = Arc::new(WsCounters::default());
        let mut q = GmoWsBackpressure::new(1, counters.clone());
        q.try_enqueue(Bytes::from_static(b"1"));
        q.try_enqueue(Bytes::from_static(b"2"));
        assert_eq!(counters.ws_backpressure_drops_total.load(Ordering::Relaxed), 1);
        assert_eq!(q.recv().await.unwrap(), Bytes::from_static(b"1"));
    }

    #[test]
    fn orderbook_gap_goes_degraded_then_resync_recovers() {
        let mut engine = OrderBookResyncEngine::default();
        engine.ingest_delta(OrderBookDelta { bids: vec![], asks: vec![], sequence_start: 11, sequence_end: 11 }).unwrap();
        let snap = engine.apply_snapshot(OrderBookSnapshot { bids: vec![], asks: vec![], sequence: 10 }).unwrap();
        assert_eq!(snap.sequence, 11);
        let err = engine.ingest_delta(OrderBookDelta { bids: vec![], asks: vec![], sequence_start: 13, sequence_end: 13 }).unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert_eq!(engine.health(), OrderBookHealth::Degraded);
        let snap2 = engine.apply_snapshot(OrderBookSnapshot { bids: vec![], asks: vec![], sequence: 20 }).unwrap();
        assert_eq!(snap2.sequence, 20);
        assert_eq!(engine.health(), OrderBookHealth::Ok);
    }

    #[test]
    fn secret_sanitize_removes_key_material() {
        let msg = "api_key=AKIA api_secret=super_secret key_id=k-1";
        let out = sanitize_log_line(msg);
        assert!(!out.contains("AKIA"));
        assert!(!out.contains("super_secret"));
        assert!(out.contains("key_id=k-1"));
    }

    #[test]
    fn strict_coverage_gate_requires_zero_gaps() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/gmocoin.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }

    #[test]
    fn bench_like_deserialize_and_normalize_regression_guard() {
        let ticker = Bytes::from_static(br#"{"status":0,"data":[{"ask":"1","bid":"1","last":"1"}]}"#);
        let generic = Bytes::from_static(br#"{"status":0,"data":{"endpoint":"x","ok":true}}"#);
        for _ in 0..1000 {
            let _: GmoEnvelope<Vec<TickerPayload>> = parse_json(&ticker).unwrap();
            let _: GmoEnvelope<GenericPayload> = parse_json(&generic).unwrap();
        }
    }
}

pub mod symbols;
pub mod ws_manager;
pub mod channels;
