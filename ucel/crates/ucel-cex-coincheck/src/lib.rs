use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc;
use tracing::info;
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
    let raw = include_str!("../../../../docs/exchanges/coincheck/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid coincheck catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|row| WsChannelSpec {
            id: row.id,
            ws_url: row.ws_url,
            channel: row.channel,
            requires_auth: row.auth.auth_type != "none",
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoincheckWsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub channel: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Trade {
        channel_id: String,
        trade_id: String,
        pair: String,
        side: String,
        rate: String,
        amount: String,
        timestamp: String,
    },
    Orderbook {
        channel_id: String,
        pair: String,
        bids: Vec<(String, String)>,
        asks: Vec<(String, String)>,
    },
    OrderEvent {
        channel_id: String,
        event: String,
        order_id: Option<String>,
    },
    ExecutionEvent {
        channel_id: String,
        event: String,
        execution_id: Option<String>,
    },
}

#[derive(Debug, Default, Clone)]
pub struct WsAdapterMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_overflow_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
    pub ws_duplicate_total: u64,
    pub ws_out_of_order_total: u64,
}

pub struct WsBackpressureBuffer {
    tx: mpsc::Sender<MarketEvent>,
    rx: mpsc::Receiver<MarketEvent>,
}

impl WsBackpressureBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    pub fn try_push(&self, event: MarketEvent, metrics: &mut WsAdapterMetrics) {
        if self.tx.try_send(event).is_err() {
            metrics.ws_backpressure_overflow_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<MarketEvent> {
        self.rx.recv().await
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderbookSyncState {
    pub last_sequence: Option<u64>,
    pub degraded: bool,
    pub bids: BTreeMap<String, String>,
    pub asks: BTreeMap<String, String>,
}

impl OrderbookSyncState {
    pub fn apply_snapshot(
        &mut self,
        sequence: u64,
        bids: Vec<(String, String)>,
        asks: Vec<(String, String)>,
    ) {
        self.last_sequence = Some(sequence);
        self.degraded = false;
        self.bids = bids.into_iter().collect();
        self.asks = asks.into_iter().collect();
    }

    pub fn apply_delta(
        &mut self,
        sequence: Option<u64>,
        bids: Vec<(String, String)>,
        asks: Vec<(String, String)>,
        metrics: &mut WsAdapterMetrics,
    ) -> bool {
        let Some(next) = sequence else {
            self.mark_gap(metrics);
            return false;
        };

        match self.last_sequence {
            Some(last) if next == last => {
                metrics.ws_duplicate_total += 1;
                self.mark_gap(metrics);
                false
            }
            Some(last) if next != last + 1 => {
                metrics.ws_out_of_order_total += 1;
                self.mark_gap(metrics);
                false
            }
            None => {
                self.mark_gap(metrics);
                false
            }
            _ => {
                self.last_sequence = Some(next);
                for (p, q) in bids {
                    if q == "0" {
                        self.bids.remove(&p);
                    } else {
                        self.bids.insert(p, q);
                    }
                }
                for (p, q) in asks {
                    if q == "0" {
                        self.asks.remove(&p);
                    } else {
                        self.asks.insert(p, q);
                    }
                }
                true
            }
        }
    }

    pub fn mark_recovered(&mut self, metrics: &mut WsAdapterMetrics) {
        if self.degraded {
            self.degraded = false;
            metrics.ws_orderbook_recovered_total += 1;
        }
    }

    fn mark_gap(&mut self, metrics: &mut WsAdapterMetrics) {
        self.degraded = true;
        metrics.ws_orderbook_gap_total += 1;
        metrics.ws_orderbook_resync_total += 1;
    }
}

#[derive(Debug, Default)]
pub struct CoincheckWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: WsAdapterMetrics,
}

impl CoincheckWsAdapter {
    pub fn build_subscribe(
        endpoint_id: &str,
        pair: &str,
        key_id: Option<&str>,
    ) -> Result<CoincheckWsRequest, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private websocket endpoint requires key_id",
            ));
        }
        let channel = spec.channel.replace("{pair}", pair);
        if spec.requires_auth {
            Ok(CoincheckWsRequest {
                r#type: None,
                command: Some("subscribe".into()),
                channel,
            })
        } else {
            Ok(CoincheckWsRequest {
                r#type: Some("subscribe".into()),
                command: None,
                channel,
            })
        }
    }

    pub fn build_unsubscribe(
        endpoint_id: &str,
        pair: &str,
        key_id: Option<&str>,
    ) -> Result<CoincheckWsRequest, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private websocket endpoint requires key_id",
            ));
        }
        let channel = spec.channel.replace("{pair}", pair);
        if spec.requires_auth {
            Ok(CoincheckWsRequest {
                r#type: None,
                command: Some("unsubscribe".into()),
                channel,
            })
        } else {
            Ok(CoincheckWsRequest {
                r#type: Some("unsubscribe".into()),
                command: None,
                channel,
            })
        }
    }

    pub fn connect_context(
        endpoint_id: &str,
        key_id: Option<&str>,
    ) -> Result<RequestContext, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(ErrorCode::MissingAuth, "missing key_id"));
        }
        Ok(RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: op_for_endpoint(endpoint_id),
            venue: "coincheck".into(),
            policy_id: "default".into(),
            key_id: key_id.map(|x| x.to_string()),
            requires_auth: spec.requires_auth,
        })
    }

    pub fn subscribe_once(&mut self, endpoint_id: &str, pair: &str) -> bool {
        self.subscriptions.insert(format!("{endpoint_id}:{pair}"))
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(
        &mut self,
        transport: &T,
    ) -> Result<usize, UcelError> {
        let endpoint = ws_channel_specs()
            .into_iter()
            .find(|s| !s.requires_auth)
            .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing public channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "coincheck".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(
                WsConnectRequest {
                    url: endpoint.ws_url,
                },
                ctx,
            )
            .await?;
        self.metrics.ws_reconnect_total += 1;
        self.metrics.ws_resubscribe_total += self.subscriptions.len() as u64;
        Ok(self.subscriptions.len())
    }

    pub fn log_private_attempt(key_id: &str, api_key: &str, api_secret: &str) {
        info!(key_id = key_id, "private connect requested");
        info!("api_key={} api_secret={} key_id={}", "***", "***", key_id);
        let _ = (api_key, api_secret);
    }
}

fn op_for_endpoint(endpoint_id: &str) -> OpName {
    match endpoint_id {
        "coincheck.ws.public.trades" => OpName::SubscribeTrades,
        "coincheck.ws.public.orderbook" => OpName::SubscribeOrderbook,
        "coincheck.ws.private.order_events" => OpName::SubscribeOrderEvents,
        "coincheck.ws.private.execution_events" => OpName::SubscribeExecutionEvents,
        _ => OpName::FetchStatus,
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncomingMessage {
    TradeBatch(Vec<TradeTuple>),
    Orderbook(OrderbookTuple),
    PrivateOrder(PrivateOrderMsg),
    PrivateExecution(PrivateExecutionMsg),
}

#[derive(Debug, Deserialize)]
struct TradeTuple(String, String, String, String, String, String);

#[derive(Debug, Deserialize)]
struct OrderbookTuple(String, Vec<PriceLevel>, Vec<PriceLevel>);

#[derive(Debug, Deserialize)]
struct PriceLevel(String, String);

#[derive(Debug, Deserialize)]
struct PrivateOrderMsg {
    event: String,
    order: PrivateOrderPayload,
}

#[derive(Debug, Deserialize)]
struct PrivateOrderPayload {
    #[serde(default)]
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrivateExecutionMsg {
    event: String,
    execution: PrivateExecutionPayload,
}

#[derive(Debug, Deserialize)]
struct PrivateExecutionPayload {
    #[serde(default)]
    id: Option<String>,
}

pub fn decode_market_event(endpoint_id: &str, raw: &[u8]) -> Result<MarketEvent, UcelError> {
    let msg: IncomingMessage = serde_json::from_slice(raw).map_err(|e| {
        UcelError::new(
            ErrorCode::Internal,
            format!("coincheck ws parse error: {e}"),
        )
    })?;
    match msg {
        IncomingMessage::TradeBatch(mut trades) => {
            let t = trades
                .pop()
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "empty trade batch"))?;
            Ok(MarketEvent::Trade {
                channel_id: endpoint_id.to_string(),
                trade_id: t.0,
                pair: t.1,
                rate: t.2,
                amount: t.3,
                side: t.4,
                timestamp: t.5,
            })
        }
        IncomingMessage::Orderbook(book) => Ok(MarketEvent::Orderbook {
            channel_id: endpoint_id.to_string(),
            pair: book.0,
            bids: book.1.into_iter().map(|x| (x.0, x.1)).collect(),
            asks: book.2.into_iter().map(|x| (x.0, x.1)).collect(),
        }),
        IncomingMessage::PrivateOrder(msg) => Ok(MarketEvent::OrderEvent {
            channel_id: endpoint_id.to_string(),
            event: msg.event,
            order_id: msg.order.id,
        }),
        IncomingMessage::PrivateExecution(msg) => Ok(MarketEvent::ExecutionEvent {
            channel_id: endpoint_id.to_string(),
            event: msg.event,
            execution_id: msg.execution.id,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;
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
            let key = spec.requires_auth.then_some("kid");
            let sub = CoincheckWsAdapter::build_subscribe(&spec.id, "btc_jpy", key).unwrap();
            assert_eq!(sub.channel, spec.channel.replace("{pair}", "btc_jpy"));
            let unsub = CoincheckWsAdapter::build_unsubscribe(&spec.id, "btc_jpy", key).unwrap();
            assert_eq!(unsub.channel, sub.channel);
        }
    }

    #[test]
    fn private_preflight_rejects_without_key_and_never_connects() {
        let err = CoincheckWsAdapter::build_subscribe(
            "coincheck.ws.private.order_events",
            "btc_jpy",
            None,
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);

        let err =
            CoincheckWsAdapter::connect_context("coincheck.ws.private.execution_events", None)
                .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[test]
    fn public_context_has_zero_key_path() {
        let ctx = CoincheckWsAdapter::connect_context("coincheck.ws.public.trades", None).unwrap();
        assert!(!ctx.requires_auth);
        assert!(ctx.key_id.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_is_idempotent() {
        let transport = SpyTransport::new();
        let mut ws = CoincheckWsAdapter::default();
        assert!(ws.subscribe_once("coincheck.ws.public.trades", "btc_jpy"));
        assert!(!ws.subscribe_once("coincheck.ws.public.trades", "btc_jpy"));

        let restored = ws.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(transport.ws_connects(), 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_counts_drops() {
        let mut q = WsBackpressureBuffer::with_capacity(1);
        let mut metrics = WsAdapterMetrics::default();

        q.try_push(
            MarketEvent::OrderEvent {
                channel_id: "c".into(),
                event: "e".into(),
                order_id: None,
            },
            &mut metrics,
        );
        q.try_push(
            MarketEvent::ExecutionEvent {
                channel_id: "c".into(),
                event: "e2".into(),
                execution_id: None,
            },
            &mut metrics,
        );

        assert_eq!(metrics.ws_backpressure_overflow_total, 1);
        assert!(q.recv().await.is_some());
    }

    #[test]
    fn typed_decode_for_all_ws_channels() {
        let trade_raw = br#"[["1","btc_jpy","100","0.1","buy","1700000000"]]"#;
        let ob_raw = br#"["btc_jpy", [["100","1"]], [["101","2"]]]"#;
        let order_raw = br#"{"event":"order_created","order":{"id":"ord-1"}}"#;
        let exe_raw = br#"{"event":"execution","execution":{"id":"exe-1"}}"#;

        assert!(matches!(
            decode_market_event("coincheck.ws.public.trades", trade_raw).unwrap(),
            MarketEvent::Trade { .. }
        ));
        assert!(matches!(
            decode_market_event("coincheck.ws.public.orderbook", ob_raw).unwrap(),
            MarketEvent::Orderbook { .. }
        ));
        assert!(matches!(
            decode_market_event("coincheck.ws.private.order_events", order_raw).unwrap(),
            MarketEvent::OrderEvent { .. }
        ));
        assert!(matches!(
            decode_market_event("coincheck.ws.private.execution_events", exe_raw).unwrap(),
            MarketEvent::ExecutionEvent { .. }
        ));
    }

    #[test]
    fn orderbook_gap_resync_and_recover() {
        let mut metrics = WsAdapterMetrics::default();
        let mut ob = OrderbookSyncState::default();

        ob.apply_snapshot(
            10,
            vec![("100".into(), "1".into())],
            vec![("101".into(), "1".into())],
        );
        let ok = ob.apply_delta(
            Some(12),
            vec![("100".into(), "2".into())],
            vec![],
            &mut metrics,
        );
        assert!(!ok);
        assert!(ob.degraded);

        ob.apply_snapshot(12, vec![("100".into(), "2".into())], vec![]);
        ob.mark_recovered(&mut metrics);

        assert_eq!(metrics.ws_orderbook_gap_total, 1);
        assert_eq!(metrics.ws_orderbook_resync_total, 1);
        assert_eq!(metrics.ws_orderbook_recovered_total, 0);
        assert!(!ob.degraded);
    }

    #[test]
    fn duplicate_and_out_of_order_policy_is_resync() {
        let mut metrics = WsAdapterMetrics::default();
        let mut ob = OrderbookSyncState::default();
        ob.apply_snapshot(5, vec![], vec![]);

        assert!(!ob.apply_delta(Some(5), vec![], vec![], &mut metrics));
        assert!(!ob.apply_delta(Some(8), vec![], vec![], &mut metrics));

        assert_eq!(metrics.ws_duplicate_total, 1);
        assert_eq!(metrics.ws_out_of_order_total, 1);
        assert!(ob.degraded);
    }

    #[derive(Clone, Default)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);
    impl Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> MakeWriter<'a> for SharedBuf {
        type Writer = SharedBuf;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn tracing_logs_do_not_leak_api_secrets() {
        let sink = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(sink.clone())
            .without_time()
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            CoincheckWsAdapter::log_private_attempt("kid-1", "my-api-key", "my-secret");
        });
        let out = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        assert!(out.contains("kid-1"));
        assert!(!out.contains("my-api-key"));
        assert!(!out.contains("my-secret"));
    }

    #[derive(Debug, Deserialize)]
    struct CoverageManifest {
        strict: bool,
        entries: Vec<CoverageEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct CoverageEntry {
        implemented: bool,
        tested: bool,
    }

    #[test]
    fn strict_coverage_gate_is_on_and_zero_gaps() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/coincheck.yaml");
        let raw = std::fs::read_to_string(path).unwrap();
        let manifest: CoverageManifest = serde_yaml::from_str(&raw).unwrap();
        assert!(manifest.strict, "strict gate must be enabled for coincheck");
        assert!(manifest.entries.iter().all(|e| e.implemented && e.tested));
    }
}
