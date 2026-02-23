use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    ws_channels: Vec<CatalogWsEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogWsEntry {
    id: String,
    ws_url: String,
    channel: String,
    auth: CatalogAuth,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogAuth {
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: String,
    pub ws_url: String,
    pub channel: String,
    pub requires_auth: bool,
}

pub fn ws_channel_specs() -> Vec<WsChannelSpec> {
    let raw = include_str!("../../../../docs/exchanges/bybit/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bybit catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|w| WsChannelSpec {
            id: w.id,
            ws_url: w.ws_url,
            channel: w.channel,
            requires_auth: w.auth.auth_type != "none",
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct WsAdapterMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_overflow_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BybitWsRequest {
    pub op: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedWsEvent {
    pub channel: String,
    pub topic: Option<String>,
    pub kind: String,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BybitEnvelope {
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    data: Option<BybitData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BybitData {
    Book(BybitOrderbookData),
    Trade(Vec<BybitTradeData>),
    Generic(BybitMapData),
}

#[derive(Debug, Clone, Deserialize)]
struct BybitOrderbookData {
    #[serde(default)]
    s: Option<String>,
    #[serde(default)]
    u: Option<u64>,
    #[serde(default)]
    seq: Option<u64>,
    #[serde(default)]
    b: Vec<(String, String)>,
    #[serde(default)]
    a: Vec<(String, String)>,
}

#[derive(Debug, Clone, Deserialize)]
struct BybitTradeData {
    #[serde(default)]
    s: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BybitMapData {
    #[serde(flatten)]
    fields: BTreeMap<String, String>,
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
    pub bids: BTreeMap<String, String>,
    pub asks: BTreeMap<String, String>,
    pub degraded: bool,
}

impl OrderBookSyncState {
    pub(crate) fn apply_snapshot(&mut self, data: &BybitOrderbookData) {
        self.sequence = data.u.or(data.seq);
        self.degraded = false;
        self.bids = data.b.iter().cloned().collect();
        self.asks = data.a.iter().cloned().collect();
    }

    pub(crate) fn apply_delta(
        &mut self,
        data: &BybitOrderbookData,
        metrics: &mut WsAdapterMetrics,
    ) {
        let next = data.u.or(data.seq);
        let expected_next = self.sequence.map(|s| s + 1);
        if expected_next.is_none() || next != expected_next {
            self.degraded = true;
            metrics.ws_orderbook_gap_total += 1;
            metrics.ws_orderbook_resync_total += 1;
            return;
        }
        self.sequence = next;
        for (price, qty) in &data.b {
            if qty == "0" {
                self.bids.remove(price);
            } else {
                self.bids.insert(price.clone(), qty.clone());
            }
        }
        for (price, qty) in &data.a {
            if qty == "0" {
                self.asks.remove(price);
            } else {
                self.asks.insert(price.clone(), qty.clone());
            }
        }
    }

    pub fn mark_recovered(&mut self, metrics: &mut WsAdapterMetrics) {
        if self.degraded {
            self.degraded = false;
            metrics.ws_orderbook_recovered_total += 1;
        }
    }
}

#[derive(Debug, Default)]
pub struct BybitWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: WsAdapterMetrics,
}

impl BybitWsAdapter {
    pub fn build_subscribe(
        endpoint_id: &str,
        symbol: &str,
        interval_or_depth: Option<&str>,
        credentials: Option<(&str, &str)>,
    ) -> Result<BybitWsRequest, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && credentials.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private websocket endpoint requires credentials",
            ));
        }

        let arg = spec
            .channel
            .replace("{symbol}", symbol)
            .replace("{depth}", interval_or_depth.unwrap_or("1"))
            .replace("{interval}", interval_or_depth.unwrap_or("1"))
            .replace("{coin}", symbol);

        Ok(BybitWsRequest {
            op: "subscribe".into(),
            args: vec![arg],
        })
    }

    pub fn build_unsubscribe(
        endpoint_id: &str,
        symbol: &str,
        interval_or_depth: Option<&str>,
    ) -> Result<BybitWsRequest, UcelError> {
        let mut req =
            Self::build_subscribe(endpoint_id, symbol, interval_or_depth, Some(("x", "y")))?;
        req.op = "unsubscribe".into();
        Ok(req)
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
            venue: "bybit".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(
                WsConnectRequest {
                    url: "wss://stream.bybit.com/v5/public/linear".into(),
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
    let msg: BybitEnvelope = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("ws json parse error: {e}")))?;
    let symbol = match msg.data {
        Some(BybitData::Book(b)) => b.s,
        Some(BybitData::Trade(mut t)) => t.pop().and_then(|x| x.s),
        Some(BybitData::Generic(g)) => g.fields.get("symbol").cloned(),
        None => None,
    };
    Ok(NormalizedWsEvent {
        channel: endpoint_id.to_string(),
        topic: msg.topic,
        symbol,
        kind: if msg.success == Some(false) {
            "error".into()
        } else {
            msg.kind.unwrap_or_else(|| "update".into())
        },
    })
}

pub fn scrub_secrets(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.starts_with("api_key=") {
                "api_key=***".to_string()
            } else if part.starts_with("api_secret=") {
                "api_secret=***".to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct CoverageManifest {
    strict: bool,
    entries: Vec<CoverageEntry>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct CoverageEntry {
    implemented: bool,
    tested: bool,
}

#[cfg(test)]
fn load_coverage_manifest(path: &std::path::Path) -> Result<CoverageManifest, UcelError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("read manifest: {e}")))?;
    serde_yaml::from_str(&raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("parse manifest: {e}")))
}

#[cfg(test)]
fn evaluate_coverage_gate(manifest: &CoverageManifest) -> Vec<&'static str> {
    let mut gaps = Vec::new();
    if manifest.entries.iter().any(|e| !e.implemented) {
        gaps.push("implemented");
    }
    if manifest.entries.iter().any(|e| !e.tested) {
        gaps.push("tested");
    }
    gaps
}

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
        fn ws_connects(&self) -> usize {
            self.ws_connects.load(Ordering::Relaxed)
        }
    }
    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Err(UcelError::new(ErrorCode::NotSupported, "http unused"))
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
    fn all_catalog_ws_channels_have_subscribe_unsubscribe() {
        for spec in ws_channel_specs() {
            let creds = if spec.requires_auth {
                Some(("key", "sec"))
            } else {
                None
            };
            let sub =
                BybitWsAdapter::build_subscribe(&spec.id, "BTCUSDT", Some("1"), creds).unwrap();
            assert_eq!(sub.op, "subscribe");
            let unsub = BybitWsAdapter::build_unsubscribe(&spec.id, "BTCUSDT", Some("1")).unwrap();
            assert_eq!(unsub.op, "unsubscribe");
        }
    }

    #[test]
    fn private_preflight_rejects_missing_credentials_and_no_connect() {
        let err = BybitWsAdapter::build_subscribe(
            "bybit.private.ws.private.order",
            "BTCUSDT",
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_is_idempotent() {
        let transport = SpyTransport::new();
        let mut ws = BybitWsAdapter::default();
        assert!(ws.subscribe_once("bybit.public.ws.public.trade", "BTCUSDT"));
        assert!(!ws.subscribe_once("bybit.public.ws.public.trade", "BTCUSDT"));
        let restored = ws.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(transport.ws_connects(), 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 1);
    }

    #[test]
    fn ws_messages_are_typed_and_normalized() {
        let msg = r#"{"topic":"publicTrade.BTCUSDT","type":"snapshot","data":[{"s":"BTCUSDT"}]}"#;
        let n = normalize_ws_event("bybit.public.ws.public.trade", msg).unwrap();
        assert_eq!(n.symbol.as_deref(), Some("BTCUSDT"));
        assert_eq!(n.kind, "snapshot");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_is_bounded_and_counts_drop() {
        let mut metrics = WsAdapterMetrics::default();
        let mut q = WsBackpressureBuffer::with_capacity(1);
        q.try_push(
            NormalizedWsEvent {
                channel: "c".into(),
                topic: None,
                kind: "u".into(),
                symbol: Some("BTCUSDT".into()),
            },
            &mut metrics,
        );
        q.try_push(
            NormalizedWsEvent {
                channel: "c".into(),
                topic: None,
                kind: "u".into(),
                symbol: Some("ETHUSDT".into()),
            },
            &mut metrics,
        );
        assert_eq!(metrics.ws_backpressure_overflow_total, 1);
        assert_eq!(q.recv().await.unwrap().symbol.as_deref(), Some("BTCUSDT"));
    }

    #[test]
    fn orderbook_gap_resync_recover_flow() {
        let mut metrics = WsAdapterMetrics::default();
        let mut ob = OrderBookSyncState::default();
        ob.apply_snapshot(&BybitOrderbookData {
            s: Some("BTCUSDT".into()),
            u: Some(10),
            seq: None,
            b: vec![("100".into(), "1".into())],
            a: vec![("101".into(), "1".into())],
        });
        ob.apply_delta(
            &BybitOrderbookData {
                s: Some("BTCUSDT".into()),
                u: Some(12),
                seq: None,
                b: vec![],
                a: vec![],
            },
            &mut metrics,
        );
        assert!(ob.degraded);
        ob.apply_snapshot(&BybitOrderbookData {
            s: Some("BTCUSDT".into()),
            u: Some(12),
            seq: None,
            b: vec![("100".into(), "2".into())],
            a: vec![],
        });
        ob.mark_recovered(&mut metrics);
        assert!(!ob.degraded);
        assert_eq!(metrics.ws_orderbook_gap_total, 1);
        assert_eq!(metrics.ws_orderbook_resync_total, 1);
    }

    #[test]
    fn duplicate_or_out_of_order_is_safe_side_resync() {
        let mut metrics = WsAdapterMetrics::default();
        let mut ob = OrderBookSyncState::default();
        ob.apply_snapshot(&BybitOrderbookData {
            s: None,
            u: Some(1),
            seq: None,
            b: vec![],
            a: vec![],
        });
        ob.apply_delta(
            &BybitOrderbookData {
                s: None,
                u: Some(1),
                seq: None,
                b: vec![],
                a: vec![],
            },
            &mut metrics,
        );
        assert!(ob.degraded);
    }

    #[test]
    fn secrets_are_scrubbed_from_logs() {
        let line = "key_id=alpha api_key=hello api_secret=world";
        let scrubbed = scrub_secrets(line);
        assert!(scrubbed.contains("key_id=alpha"));
        assert!(!scrubbed.contains("hello"));
        assert!(!scrubbed.contains("world"));
    }

    #[test]
    fn strict_coverage_gate_is_on_and_has_zero_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bybit.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict gate requires zero gaps: {gaps:?}");
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
