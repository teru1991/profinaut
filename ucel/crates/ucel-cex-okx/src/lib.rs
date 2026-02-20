use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub source_url: String,
    pub path_or_doc: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct OkxRestAdapter {
    endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl OkxRestAdapter {
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

    pub fn endpoint_specs(&self) -> &[EndpointSpec] {
        &self.endpoints
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<OkxRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown endpoint: {endpoint_id}")))?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "okx".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}::{}", spec.source_url, spec.path_or_doc),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_okx_http_error(response.status, &response.body));
        }

        parse_response(&response.body)
    }
}

impl Default for OkxRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: String,
    pub visibility: String,
    pub channel_or_doc: String,
    pub source_url: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkxWsRequest {
    pub op: String,
    pub args: Vec<OkxWsArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkxWsArg {
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inst_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedWsEvent {
    pub channel_id: String,
    pub channel: String,
    pub symbol: Option<String>,
    pub event_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct OkxWsMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_drops_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
}

#[derive(Debug)]
pub struct WsBackpressureBuffer {
    tx: mpsc::Sender<NormalizedWsEvent>,
    rx: mpsc::Receiver<NormalizedWsEvent>,
}

impl WsBackpressureBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    pub fn try_push(&self, event: NormalizedWsEvent, metrics: &mut OkxWsMetrics) {
        if self.tx.try_send(event).is_err() {
            metrics.ws_backpressure_drops_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<NormalizedWsEvent> {
        self.rx.recv().await
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrderBookSyncState {
    pub seq_id: Option<u64>,
    pub degraded: bool,
    pub bids: BTreeMap<String, String>,
    pub asks: BTreeMap<String, String>,
}

impl OrderBookSyncState {
    pub fn apply_snapshot(&mut self, book: &OkxBookData, metrics: Option<&mut OkxWsMetrics>) {
        self.seq_id = book.seq_id;
        if self.degraded {
            if let Some(m) = metrics {
                m.ws_orderbook_recovered_total += 1;
            }
        }
        self.degraded = false;
        self.bids = book.bids.iter().cloned().collect();
        self.asks = book.asks.iter().cloned().collect();
    }

    pub fn apply_delta(&mut self, book: &OkxBookData, metrics: &mut OkxWsMetrics) {
        let expected = self.seq_id.map(|s| s + 1);
        if expected.is_none() || book.prev_seq_id != self.seq_id || book.seq_id != expected {
            self.degraded = true;
            metrics.ws_orderbook_gap_total += 1;
            metrics.ws_orderbook_resync_total += 1;
            return;
        }
        self.seq_id = book.seq_id;
        for (price, qty) in &book.bids {
            if qty == "0" {
                self.bids.remove(price);
            } else {
                self.bids.insert(price.clone(), qty.clone());
            }
        }
        for (price, qty) in &book.asks {
            if qty == "0" {
                self.asks.remove(price);
            } else {
                self.asks.insert(price.clone(), qty.clone());
            }
        }
    }

    pub fn mark_recovered(&mut self, metrics: &mut OkxWsMetrics) {
        if self.degraded {
            self.degraded = false;
            metrics.ws_orderbook_recovered_total += 1;
        }
    }
}

#[derive(Debug, Default)]
pub struct OkxWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: OkxWsMetrics,
}

impl OkxWsAdapter {
    pub fn ws_channel_specs() -> Vec<WsChannelSpec> {
        load_ws_channel_specs()
    }

    pub fn build_subscribe(
        channel_id: &str,
        symbol: Option<&str>,
        key_id: Option<&str>,
    ) -> Result<OkxWsRequest, UcelError> {
        let spec = load_ws_channel_specs()
            .into_iter()
            .find(|s| s.id == channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(ErrorCode::MissingAuth, "private websocket endpoint requires key_id"));
        }
        if spec.requires_auth {
            log_private_ws_preflight(key_id.unwrap_or(""));
        }
        Ok(OkxWsRequest {
            op: "subscribe".into(),
            args: vec![OkxWsArg {
                channel: spec.channel_or_doc,
                inst_id: symbol.map(|s| s.to_string()),
            }],
        })
    }

    pub fn build_unsubscribe(
        channel_id: &str,
        symbol: Option<&str>,
        key_id: Option<&str>,
    ) -> Result<OkxWsRequest, UcelError> {
        let mut req = Self::build_subscribe(channel_id, symbol, key_id)?;
        req.op = "unsubscribe".into();
        Ok(req)
    }

    pub fn subscribe_once(&mut self, channel_id: &str, symbol: Option<&str>) -> bool {
        self.subscriptions.insert(format!("{channel_id}:{}", symbol.unwrap_or("*")))
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(&mut self, transport: &T) -> Result<usize, UcelError> {
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "okx".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(WsConnectRequest { url: "wss://ws.okx.com:8443/ws/v5/public".into() }, ctx)
            .await?;
        self.metrics.ws_reconnect_total += 1;
        self.metrics.ws_resubscribe_total += self.subscriptions.len() as u64;
        Ok(self.subscriptions.len())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OkxEnvelope {
    pub code: String,
    pub msg: String,
    pub data: Vec<OkxDataRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OkxRestResponse {
    Envelope(OkxEnvelope),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OkxDataRecord {
    #[serde(flatten)]
    pub fields: BTreeMap<String, OkxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OkxField {
    Scalar(OkxScalar),
    Object(OkxDataRecord),
    Array(Vec<OkxField>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OkxScalar {
    String(String),
    Number(f64),
    Bool(bool),
    Null(()),
}

#[derive(Debug, Deserialize)]
struct OkxEnvelopeWire {
    code: String,
    #[serde(default)]
    msg: String,
    #[serde(default)]
    data: Vec<OkxDataRecord>,
}

#[derive(Debug, Deserialize)]
struct OkxWsEnvelope {
    #[serde(default)]
    arg: Option<OkxWsArg>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    data: Vec<OkxWsData>,
}

#[derive(Debug, Deserialize)]
struct OkxWsData {
    #[serde(rename = "instId", default)]
    inst_id: Option<String>,
    #[serde(rename = "seqId", default)]
    seq_id: Option<u64>,
    #[serde(rename = "prevSeqId", default)]
    prev_seq_id: Option<u64>,
    #[serde(default)]
    bids: Vec<(String, String)>,
    #[serde(default)]
    asks: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct OkxBookData {
    pub seq_id: Option<u64>,
    pub prev_seq_id: Option<u64>,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

pub fn normalize_ws_event(channel_id: &str, raw: &str) -> Result<NormalizedWsEvent, UcelError> {
    let msg: OkxWsEnvelope = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("ws json parse error: {e}")))?;
    let first = msg.data.first();
    Ok(NormalizedWsEvent {
        channel_id: channel_id.to_string(),
        channel: msg
            .arg
            .as_ref()
            .map(|a| a.channel.clone())
            .unwrap_or_else(|| "unknown".into()),
        symbol: first.and_then(|d| d.inst_id.clone()),
        event_type: msg.event.unwrap_or_else(|| {
            if msg.data.is_empty() {
                "ack".into()
            } else {
                "update".into()
            }
        }),
    })
}

pub fn parse_okx_orderbook_payload(raw: &str) -> Result<Option<OkxBookData>, UcelError> {
    let msg: OkxWsEnvelope = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("ws json parse error: {e}")))?;
    let Some(data) = msg.data.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(OkxBookData {
        seq_id: data.seq_id,
        prev_seq_id: data.prev_seq_id,
        bids: data.bids,
        asks: data.asks,
    }))
}

fn parse_response(bytes: &[u8]) -> Result<OkxRestResponse, UcelError> {
    if bytes.is_empty() {
        return Ok(OkxRestResponse::Empty);
    }
    let payload: OkxEnvelopeWire = serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;
    Ok(OkxRestResponse::Envelope(OkxEnvelope {
        code: payload.code,
        msg: payload.msg,
        data: payload.data,
    }))
}

#[derive(Debug, Deserialize)]
struct OkxErrorEnvelope {
    code: Option<String>,
    msg: Option<String>,
    data: Option<Vec<OkxErrorDetail>>,
}

#[derive(Debug, Deserialize)]
struct OkxErrorDetail {
    #[serde(rename = "sCode")]
    s_code: Option<String>,
}

pub fn map_okx_http_error(status: u16, body: &[u8]) -> UcelError {
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

    let envelope = serde_json::from_slice::<OkxErrorEnvelope>(body).ok();
    let code = envelope
        .as_ref()
        .and_then(|v| v.code.as_deref())
        .or_else(|| {
            envelope
                .as_ref()
                .and_then(|v| v.data.as_ref())
                .and_then(|items| items.first())
                .and_then(|d| d.s_code.as_deref())
        })
        .unwrap_or_default()
        .to_string();

    let mapped = match code.as_str() {
        "50011" | "50061" | "50040" => ErrorCode::RateLimited,
        "50113" | "50104" | "50101" => ErrorCode::AuthFailed,
        "50035" | "50036" => ErrorCode::PermissionDenied,
        "51000" | "51008" | "51100" | "51101" | "51131" => ErrorCode::InvalidOrder,
        _ => {
            if status == 401 {
                ErrorCode::AuthFailed
            } else if status == 403 {
                ErrorCode::PermissionDenied
            } else if status == 400 || status == 404 || status == 409 || status == 422 {
                ErrorCode::InvalidOrder
            } else {
                ErrorCode::Network
            }
        }
    };

    let message = envelope
        .and_then(|v| v.msg)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| format!("okx http error status={status} code={}", code));
    UcelError::new(mapped, message)
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogRestEntry>,
    ws_channels: Vec<CatalogWsEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogRestEntry {
    id: String,
    method: String,
    visibility: String,
    path_or_doc: String,
    source_url: String,
}

#[derive(Debug, Deserialize)]
struct CatalogWsEntry {
    id: String,
    visibility: String,
    channel_or_doc: String,
    source_url: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/okx/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid okx catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            source_url: entry.source_url,
            path_or_doc: entry.path_or_doc,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

fn load_ws_channel_specs() -> Vec<WsChannelSpec> {
    let raw = include_str!("../../../../docs/exchanges/okx/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid okx catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|entry| WsChannelSpec {
            id: entry.id,
            visibility: entry.visibility.clone(),
            channel_or_doc: entry.channel_or_doc,
            source_url: entry.source_url,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

pub fn log_private_ws_preflight(key_id: &str) {
    info!(target: "okx.auth", key_id = key_id, "private ws subscribe preflight passed");
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
    use std::io;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;
    use ucel_transport::{HttpResponse, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        calls: AtomicUsize,
        ws_connects: AtomicUsize,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                ws_connects: AtomicUsize::new(0),
                response: Mutex::new(Some(resp)),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(&self, _req: HttpRequest, _ctx: RequestContext) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.response.lock().expect("lock").take().expect("response")
        }

        async fn connect_ws(&self, _req: WsConnectRequest, _ctx: RequestContext) -> Result<WsStream, UcelError> {
            self.ws_connects.fetch_add(1, Ordering::SeqCst);
            Ok(WsStream { connected: true })
        }
    }

    #[derive(Clone, Default)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    struct GuardWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for GuardWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("writer lock").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for SharedWriter {
        type Writer = GuardWriter;

        fn make_writer(&'a self) -> Self::Writer {
            GuardWriter(self.0.clone())
        }
    }

    #[test]
    fn loads_all_catalog_rows() {
        let adapter = OkxRestAdapter::new();
        assert_eq!(adapter.endpoint_specs().len(), 4);
        assert_eq!(OkxWsAdapter::ws_channel_specs().len(), 3);
    }

    #[test]
    fn all_ws_channels_have_subscribe_and_unsubscribe() {
        for spec in OkxWsAdapter::ws_channel_specs() {
            let key = if spec.requires_auth { Some("k") } else { None };
            let sub = OkxWsAdapter::build_subscribe(&spec.id, Some("BTC-USDT"), key).unwrap();
            assert_eq!(sub.op, "subscribe");
            let unsub = OkxWsAdapter::build_unsubscribe(&spec.id, Some("BTC-USDT"), key).unwrap();
            assert_eq!(unsub.op, "unsubscribe");
        }
    }

    #[test]
    fn private_preflight_rejects_without_transport_hit() {
        let err = OkxWsAdapter::build_subscribe("okx.ws.private", Some("BTC-USDT"), None).unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_and_resubscribe_is_idempotent() {
        let transport = SpyTransport::default();
        let mut adapter = OkxWsAdapter::default();
        assert!(adapter.subscribe_once("okx.ws.public", Some("BTC-USDT")));
        assert!(!adapter.subscribe_once("okx.ws.public", Some("BTC-USDT")));
        let restored = adapter.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(adapter.metrics.ws_reconnect_total, 1);
        assert_eq!(adapter.metrics.ws_resubscribe_total, 1);
        assert_eq!(transport.ws_connects.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn typed_ws_message_normalization() {
        let raw = r#"{"arg":{"channel":"books","inst_id":"BTC-USDT"},"event":"snapshot","data":[{"instId":"BTC-USDT","seqId":10,"prevSeqId":9,"bids":[["1","1"]],"asks":[["2","1"]]}]}"#;
        let n = normalize_ws_event("okx.ws.public", raw).unwrap();
        assert_eq!(n.channel, "books");
        assert_eq!(n.event_type, "snapshot");
        assert_eq!(n.symbol.as_deref(), Some("BTC-USDT"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_counts_drops() {
        let mut metrics = OkxWsMetrics::default();
        let mut queue = WsBackpressureBuffer::with_capacity(1);
        queue.try_push(
            NormalizedWsEvent { channel_id: "okx.ws.public".into(), channel: "books".into(), symbol: Some("BTC-USDT".into()), event_type: "update".into() },
            &mut metrics,
        );
        queue.try_push(
            NormalizedWsEvent { channel_id: "okx.ws.public".into(), channel: "books".into(), symbol: Some("ETH-USDT".into()), event_type: "update".into() },
            &mut metrics,
        );
        assert_eq!(metrics.ws_backpressure_drops_total, 1);
        assert_eq!(queue.recv().await.unwrap().symbol.as_deref(), Some("BTC-USDT"));
    }

    #[test]
    fn orderbook_gap_to_resync_to_recovered_and_duplicate_policy() {
        let snapshot = parse_okx_orderbook_payload(
            r#"{"arg":{"channel":"books","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","seqId":10,"prevSeqId":9,"bids":[["1","1"]],"asks":[["2","1"]]}]}"#,
        )
        .unwrap()
        .unwrap();
        let gap = parse_okx_orderbook_payload(
            r#"{"arg":{"channel":"books","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","seqId":12,"prevSeqId":10,"bids":[],"asks":[]}]}"#,
        )
        .unwrap()
        .unwrap();
        let dup = parse_okx_orderbook_payload(
            r#"{"arg":{"channel":"books","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","seqId":10,"prevSeqId":9,"bids":[],"asks":[]}]}"#,
        )
        .unwrap()
        .unwrap();

        let mut metrics = OkxWsMetrics::default();
        let mut ob = OrderBookSyncState::default();
        ob.apply_snapshot(&snapshot, None);
        ob.apply_delta(&gap, &mut metrics);
        assert!(ob.degraded);

        ob.apply_snapshot(&snapshot, Some(&mut metrics));
        assert!(!ob.degraded);

        ob.apply_delta(&dup, &mut metrics);
        assert!(ob.degraded);
        assert_eq!(metrics.ws_orderbook_gap_total, 2);
        assert_eq!(metrics.ws_orderbook_resync_total, 2);
        assert_eq!(metrics.ws_orderbook_recovered_total, 1);
    }

    #[test]
    fn tracing_capture_has_no_secret_leak() {
        let shared = SharedWriter::default();
        let captured = shared.0.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(shared)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            log_private_ws_preflight("kid-001");
        });

        let output = String::from_utf8(captured.lock().expect("capture lock").clone()).expect("utf8");
        assert!(output.contains("kid-001"));
        assert!(!output.contains("api_key"));
        assert!(!output.contains("api_secret"));
    }

    #[test]
    fn strict_coverage_gate_is_on_and_has_no_gaps() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/okx.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict gate requires zero gaps: {gaps:?}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_test_all_ids() {
        let adapter = OkxRestAdapter::new();
        for spec in adapter.endpoint_specs() {
            let transport = SpyTransport::with_response(Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(br#"{"code":"0","msg":"","data":[]}"#),
            }));
            let out = adapter.execute_rest(&transport, &spec.id, None, Some("k".into())).await;
            assert!(out.is_ok(), "id={} should parse fixture", spec.id);
        }
    }
}
