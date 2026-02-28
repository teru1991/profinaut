use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use ucel_core::{
    ErrorCode, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError,
};
use ucel_transport::{
    enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport, WsConnectRequest,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsChannelSpec {
    pub id: String,
    pub ws_url: String,
    pub subscribe_template: String,
    pub unsubscribe_template: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HtxRestResponse {
    pub status: Option<String>,
    pub ch: Option<String>,
    pub ts: Option<u64>,
    pub data: Option<Vec<HtxDataItem>>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, HtxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HtxDataItem {
    #[serde(flatten)]
    pub fields: BTreeMap<String, HtxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum HtxField {
    String(String),
    Number(f64),
    Bool(bool),
    Object(BTreeMap<String, HtxField>),
    Array(Vec<HtxField>),
    Null(()),
}

#[derive(Clone)]
pub struct HtxRestAdapter {
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl HtxRestAdapter {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            endpoints: Arc::new(load_endpoint_specs()),
            http_client,
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn from_specs(specs: Vec<EndpointSpec>) -> Self {
        let mut adapter = Self::new();
        adapter.endpoints = Arc::new(specs);
        adapter
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<HtxRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|e| e.id == endpoint_id)
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
            venue: "htx".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}{}", spec.base_url, spec.path),
            body,
        };

        let resp = transport.send_http(req, ctx).await?;
        if resp.status >= 400 {
            return Err(map_htx_http_error(resp.status, &resp.body));
        }

        parse_json(&resp.body)
    }
}

impl Default for HtxRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub symbol: Option<String>,
    pub contract_code: Option<String>,
    pub topic: Option<String>,
    pub channel: Option<String>,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsCommand {
    pub payload: String,
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
    pub ws_orderbook_resync_total: AtomicU64,
}

pub struct HtxBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}

impl HtxBackpressure {
    pub fn new(capacity: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderBookHealth {
    #[default]
    Recovered,
    Degraded,
}

#[derive(Debug, Default)]
pub struct OrderBookResyncEngine {
    buffered_deltas: VecDeque<OrderBookDelta>,
    expected_next: Option<u64>,
    health: OrderBookHealth,
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
                self.expected_next = None;
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "snapshot/delta mismatch; resync required",
                ));
            }
            merge_levels(&mut snapshot.bids, delta.bids, true);
            merge_levels(&mut snapshot.asks, delta.asks, false);
            snapshot.sequence = delta.sequence_end;
            self.expected_next = Some(snapshot.sequence + 1);
        }
        self.health = OrderBookHealth::Recovered;
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

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Trade(TradeEvent),
    OrderBookDelta(OrderBookDelta),
    PrivateTopic {
        op: String,
        topic: Option<String>,
    },
    Heartbeat {
        ping: u64,
    },
    Generic {
        channel: Option<String>,
        event: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct SpotWsMsg {
    ch: Option<String>,
    tick: Option<SpotTick>,
    ts: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SpotTick {
    #[serde(default)]
    bids: Vec<(f64, f64)>,
    #[serde(default)]
    asks: Vec<(f64, f64)>,
    price: Option<f64>,
    amount: Option<f64>,
    direction: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PrivateWsMsg {
    op: String,
    topic: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CommonWsMsg {
    ping: Option<u64>,
    action: Option<String>,
    ch: Option<String>,
    status: Option<String>,
}

#[derive(Clone)]
pub struct HtxWsAdapter {
    channels: Arc<Vec<WsChannelSpec>>,
    subscriptions: Arc<Mutex<HashSet<WsSubscription>>>,
    counters: Arc<WsCounters>,
}

impl HtxWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            channels: Arc::new(load_ws_channel_specs()),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            counters,
        }
    }

    pub fn from_specs(specs: Vec<WsChannelSpec>, counters: Arc<WsCounters>) -> Self {
        Self {
            channels: Arc::new(specs),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            counters,
        }
    }

    pub fn channel_specs(&self) -> &[WsChannelSpec] {
        &self.channels
    }

    pub async fn subscribe<T: Transport>(
        &self,
        transport: &T,
        sub: WsSubscription,
    ) -> Result<bool, UcelError> {
        let spec = self
            .channels
            .iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeTrades,
            venue: "htx".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth {
                sub.key_id.clone()
            } else {
                None
            },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        transport
            .connect_ws(
                WsConnectRequest {
                    url: spec.ws_url.clone(),
                },
                ctx,
            )
            .await?;

        let mut guard = self.subscriptions.lock().await;
        Ok(guard.insert(sub))
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
            let spec = self
                .channels
                .iter()
                .find(|s| s.id == sub.channel_id)
                .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
            let ctx = RequestContext {
                trace_id: Uuid::new_v4().to_string(),
                request_id: Uuid::new_v4().to_string(),
                run_id: Uuid::new_v4().to_string(),
                op: OpName::SubscribeTrades,
                venue: "htx".into(),
                policy_id: "default".into(),
                key_id: if spec.requires_auth {
                    sub.key_id.clone()
                } else {
                    None
                },
                requires_auth: spec.requires_auth,
            };
            transport
                .connect_ws(
                    WsConnectRequest {
                        url: spec.ws_url.clone(),
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

    pub fn build_subscribe_command(&self, sub: &WsSubscription) -> Result<WsCommand, UcelError> {
        self.build_command(sub, true)
    }

    pub fn build_unsubscribe_command(&self, sub: &WsSubscription) -> Result<WsCommand, UcelError> {
        self.build_command(sub, false)
    }

    fn build_command(&self, sub: &WsSubscription, subscribe: bool) -> Result<WsCommand, UcelError> {
        let spec = self
            .channels
            .iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        let template = if subscribe {
            &spec.subscribe_template
        } else {
            &spec.unsubscribe_template
        };
        let payload = template
            .replace("$symbol", sub.symbol.as_deref().unwrap_or("btcusdt"))
            .replace(
                "$contract_code",
                sub.contract_code.as_deref().unwrap_or("BTC-USDT"),
            )
            .replace("$topic", sub.topic.as_deref().unwrap_or("depth.step0"))
            .replace(
                "$channel",
                sub.channel
                    .as_deref()
                    .unwrap_or("market.btcusdt.kline.1min"),
            );
        Ok(WsCommand { payload })
    }

    pub fn parse_market_event(
        &self,
        channel_id: &str,
        body: &Bytes,
    ) -> Result<MarketEvent, UcelError> {
        if channel_id.contains("private") {
            let msg: PrivateWsMsg = parse_json(body)?;
            return Ok(MarketEvent::PrivateTopic {
                op: msg.op,
                topic: msg.topic,
            });
        }

        let common: CommonWsMsg = parse_json(body)?;
        if let Some(ping) = common.ping {
            return Ok(MarketEvent::Heartbeat { ping });
        }
        if common.action.is_some() || common.status.is_some() {
            return Ok(MarketEvent::Generic {
                channel: common.ch,
                event: common.status.unwrap_or_else(|| "ok".to_string()),
            });
        }

        let msg: SpotWsMsg = parse_json(body)?;
        if let Some(tick) = msg.tick {
            if tick.price.is_some() {
                return Ok(MarketEvent::Trade(TradeEvent {
                    trade_id: format!(
                        "{}:{}",
                        msg.ch.clone().unwrap_or_default(),
                        msg.ts.unwrap_or_default()
                    ),
                    price: tick.price.unwrap_or_default(),
                    qty: tick.amount.unwrap_or_default(),
                    side: tick.direction.unwrap_or_else(|| "unknown".to_string()),
                }));
            }
            let bids = tick
                .bids
                .into_iter()
                .map(|(price, qty)| OrderBookLevel { price, qty })
                .collect::<Vec<_>>();
            let asks = tick
                .asks
                .into_iter()
                .map(|(price, qty)| OrderBookLevel { price, qty })
                .collect::<Vec<_>>();
            return Ok(MarketEvent::OrderBookDelta(OrderBookDelta {
                bids,
                asks,
                sequence_start: msg.ts.unwrap_or_default(),
                sequence_end: msg.ts.unwrap_or_default(),
            }));
        }

        Ok(MarketEvent::Generic {
            channel: msg.ch,
            event: "generic".to_string(),
        })
    }

    pub fn mark_resync(&self) {
        self.counters
            .ws_orderbook_resync_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn counters(&self) -> Arc<WsCounters> {
        self.counters.clone()
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct HtxErrorEnvelope {
    #[serde(rename = "err-code")]
    err_code: Option<String>,
    #[serde(rename = "error-code")]
    error_code: Option<String>,
    code: Option<String>,
    #[serde(rename = "status")]
    status_text: Option<String>,
}

pub fn map_htx_http_error(status: u16, body: &[u8]) -> UcelError {
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

    let envelope = serde_json::from_slice::<HtxErrorEnvelope>(body).ok();
    let code = envelope
        .as_ref()
        .and_then(|e| e.err_code.as_deref())
        .or_else(|| envelope.as_ref().and_then(|e| e.error_code.as_deref()))
        .or_else(|| envelope.as_ref().and_then(|e| e.code.as_deref()))
        .or_else(|| envelope.as_ref().and_then(|e| e.status_text.as_deref()))
        .unwrap_or_default()
        .to_ascii_uppercase();

    if status == 401 || status == 407 || code.contains("AUTH") || code.contains("SIGN") {
        return UcelError::new(
            ErrorCode::AuthFailed,
            format!("htx status={status} code={code}"),
        );
    }
    if status == 403 || code.contains("PERMISSION") || code.contains("FORBIDDEN") {
        return UcelError::new(
            ErrorCode::PermissionDenied,
            format!("htx status={status} code={code}"),
        );
    }
    if status == 400 || status == 404 || status == 409 || status == 422 {
        return UcelError::new(
            ErrorCode::InvalidOrder,
            format!("htx status={status} code={code}"),
        );
    }

    UcelError::new(
        ErrorCode::Network,
        format!("htx status={status} code={code}"),
    )
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
    ws_channels: Vec<CatalogWsEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    base_url: String,
    path: String,
    visibility: String,
}

#[derive(Debug, Deserialize)]
struct CatalogTemplate {
    template: String,
}

#[derive(Debug, Deserialize)]
struct CatalogWsEntry {
    id: String,
    ws_url: String,
    subscribe: CatalogTemplate,
    unsubscribe: CatalogTemplate,
    visibility: String,
}

fn load_catalog() -> Catalog {
    let raw = include_str!("../../../../docs/exchanges/htx/catalog.json");
    serde_json::from_str(raw).expect("valid htx catalog")
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    load_catalog()
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            base_url: entry.base_url,
            path: entry.path,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

fn load_ws_channel_specs() -> Vec<WsChannelSpec> {
    load_catalog()
        .ws_channels
        .into_iter()
        .map(|entry| WsChannelSpec {
            id: entry.id,
            ws_url: entry.ws_url,
            subscribe_template: entry.subscribe.template,
            unsubscribe_template: entry.unsubscribe.template,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

pub fn log_private_ws_auth_attempt(key_id: Option<&str>, _api_key: &str, _api_secret: &str) {
    tracing::info!(key_id = ?key_id, "htx private ws auth attempt");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_catalog_rows() {
        let adapter = HtxRestAdapter::new();
        assert_eq!(adapter.endpoints.len(), 13);

        let ws = HtxWsAdapter::new(Arc::new(WsCounters::default()));
        assert_eq!(ws.channel_specs().len(), 9);
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
