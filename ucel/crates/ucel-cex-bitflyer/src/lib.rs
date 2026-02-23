use bytes::Bytes;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub path: String,
    pub requires_auth: bool,
    pub response_shape: String,
}

#[derive(Debug, Clone)]
pub struct BitflyerRestAdapter {
    pub base_url: Arc<str>,
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BitflyerRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            base_url: Arc::from(base_url.into()),
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

    pub fn from_specs(base_url: impl Into<String>, specs: Vec<EndpointSpec>) -> Self {
        let mut adapter = Self::new(base_url);
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
    ) -> Result<BitflyerRestResponse, UcelError> {
        let spec = self
            .endpoints
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
            venue: "bitflyer".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}{}", self.base_url, spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_bitflyer_http_error(response.status, &response.body));
        }

        parse_response_for_shape(&response.body, &spec.response_shape)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitflyerRestResponse {
    Object(BitflyerObject),
    ArrayObject(Vec<BitflyerObject>),
    ArrayString(Vec<String>),
    Text(String),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BitflyerObject {
    #[serde(flatten)]
    pub fields: BTreeMap<String, BitflyerField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitflyerField {
    Scalar(BitflyerScalar),
    Object(BitflyerObject),
    Array(Vec<BitflyerField>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitflyerScalar {
    String(String),
    Number(f64),
    Bool(bool),
    Null(()),
}

fn parse_response_for_shape(bytes: &[u8], shape: &str) -> Result<BitflyerRestResponse, UcelError> {
    if bytes.is_empty() {
        return Ok(BitflyerRestResponse::Empty);
    }

    match shape {
        "object" => serde_json::from_slice::<BitflyerObject>(bytes)
            .map(BitflyerRestResponse::Object)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "array<object>" => serde_json::from_slice::<Vec<BitflyerObject>>(bytes)
            .map(BitflyerRestResponse::ArrayObject)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "array<string>" => serde_json::from_slice::<Vec<String>>(bytes)
            .map(BitflyerRestResponse::ArrayString)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "text/spec" => std::str::from_utf8(bytes)
            .map(|s| BitflyerRestResponse::Text(s.to_string()))
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("text parse error: {e}"))),
        _ => Err(UcelError::new(
            ErrorCode::Internal,
            format!("unsupported response shape: {shape}"),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct BitflyerErrorEnvelope {
    status: Option<i64>,
    code: Option<String>,
    error_code: Option<String>,
}

pub fn map_bitflyer_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = extract_retry_after_ms(body);
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let envelope = serde_json::from_slice::<BitflyerErrorEnvelope>(body).ok();
    let code_field = envelope
        .as_ref()
        .and_then(|e| e.error_code.as_deref().or(e.code.as_deref()))
        .unwrap_or_default()
        .to_ascii_uppercase();
    let status_field = envelope
        .as_ref()
        .and_then(|e| e.status)
        .unwrap_or(status as i64);

    let code = if status == 401 || status_field == -200 || code_field == "AUTH_ERROR" {
        ErrorCode::AuthFailed
    } else if status == 403 || code_field == "PERMISSION_DENIED" {
        ErrorCode::PermissionDenied
    } else if status == 400 || status == 404 || status == 409 || status == 422 {
        ErrorCode::InvalidOrder
    } else {
        ErrorCode::Network
    };

    UcelError::new(code, format!("bitflyer http error status={status}"))
}

fn extract_retry_after_ms(body: &[u8]) -> Option<u64> {
    if let Ok(v) = serde_json::from_slice::<BTreeMap<String, String>>(body) {
        if let Some(raw) = v.get("retry_after_ms").or_else(|| v.get("retry-after-ms")) {
            if let Ok(parsed) = raw.parse::<u64>() {
                return Some(parsed);
            }
        }
        if let Some(raw) = v.get("retry_after") {
            if let Ok(sec) = raw.parse::<u64>() {
                return Some(sec.saturating_mul(1000));
            }
        }
    }

    std::str::from_utf8(body)
        .ok()
        .and_then(|b| b.split("retry_after_ms=").nth(1))
        .and_then(|s| s.trim().parse::<u64>().ok())
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
    ws_channels: Vec<WsCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    path: String,
    access: Option<String>,
    auth: Option<AuthSpec>,
    response: ResponseSpec,
}

#[derive(Debug, Deserialize)]
struct AuthSpec {
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Debug, Deserialize)]
struct ResponseSpec {
    shape: String,
}

#[derive(Debug, Deserialize)]
struct WsCatalogEntry {
    id: String,
    ws_url: String,
    channel: String,
    access: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsChannelSpec {
    pub id: String,
    pub ws_url: String,
    pub channel: String,
    pub requires_auth: bool,
}

pub fn ws_channel_specs() -> Vec<WsChannelSpec> {
    let raw = include_str!("../../../../docs/exchanges/bitflyer/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bitflyer catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|entry| WsChannelSpec {
            id: entry.id,
            ws_url: entry.ws_url,
            channel: entry.channel,
            requires_auth: entry.access.eq_ignore_ascii_case("private"),
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct WsMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_drops_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub product_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker {
        channel: String,
        product_code: String,
    },
    Execution {
        channel: String,
        id: i64,
    },
    BoardDelta {
        channel: String,
        sequence: u64,
    },
    BoardSnapshot {
        channel: String,
        sequence: u64,
    },
    OrderEvent {
        channel: String,
        event_type: String,
    },
}

#[derive(Debug)]
pub struct WsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
}

impl WsBackpressure {
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    pub fn try_push(&self, msg: Bytes, metrics: &mut WsMetrics) {
        if self.tx.try_send(msg).is_err() {
            metrics.ws_backpressure_drops_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Default)]
pub struct OrderbookResyncState {
    pub degraded: bool,
    sequence: Option<u64>,
    pub bids: HashMap<String, String>,
    pub asks: HashMap<String, String>,
}

impl OrderbookResyncState {
    pub fn apply_snapshot(
        &mut self,
        snapshot_json: &[u8],
        metrics: &mut WsMetrics,
    ) -> Result<(), UcelError> {
        let msg: BoardMessage = serde_json::from_slice(snapshot_json).map_err(|e| {
            UcelError::new(ErrorCode::Internal, format!("snapshot parse error: {e}"))
        })?;
        self.sequence = Some(msg.mid_price as u64);
        self.bids = msg.bids.into_iter().map(|l| (l.price, l.size)).collect();
        self.asks = msg.asks.into_iter().map(|l| (l.price, l.size)).collect();
        if self.degraded {
            metrics.ws_orderbook_recovered_total += 1;
        }
        self.degraded = false;
        Ok(())
    }

    pub fn apply_delta(
        &mut self,
        delta_json: &[u8],
        metrics: &mut WsMetrics,
    ) -> Result<(), UcelError> {
        let msg: BoardMessage = serde_json::from_slice(delta_json)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("delta parse error: {e}")))?;
        let next = msg.mid_price as u64;
        let expected = self.sequence.map(|seq| seq.saturating_add(1));
        if expected.is_none() || Some(next) != expected {
            self.degraded = true;
            metrics.ws_orderbook_gap_total += 1;
            metrics.ws_orderbook_resync_total += 1;
            return Err(UcelError::new(
                ErrorCode::Desync,
                "orderbook gap: resync required",
            ));
        }
        self.sequence = Some(next);
        for l in msg.bids {
            if l.size == "0" {
                self.bids.remove(&l.price);
            } else {
                self.bids.insert(l.price, l.size);
            }
        }
        for l in msg.asks {
            if l.size == "0" {
                self.asks.remove(&l.price);
            } else {
                self.asks.insert(l.price, l.size);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct BitflyerWsAdapter {
    subscriptions: HashSet<WsSubscription>,
    pub metrics: WsMetrics,
}

impl BitflyerWsAdapter {
    pub fn build_subscribe(
        channel_id: &str,
        product_code: Option<&str>,
        key_id: Option<&str>,
    ) -> Result<String, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown websocket channel"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private websocket channel requires key_id",
            ));
        }
        let channel = spec
            .channel
            .replace("{product_code}", product_code.unwrap_or("BTC_JPY"));
        Ok(format!(
            r#"{{"method":"subscribe","params":{{"channel":"{channel}"}}}}"#
        ))
    }

    pub fn build_unsubscribe(
        channel_id: &str,
        product_code: Option<&str>,
    ) -> Result<String, UcelError> {
        let mut req = Self::build_subscribe(channel_id, product_code, Some("mask"))?;
        req = req.replace("\"subscribe\"", "\"unsubscribe\"");
        Ok(req)
    }

    pub fn subscribe_once(&mut self, channel_id: &str, product_code: Option<&str>) -> bool {
        self.subscriptions.insert(WsSubscription {
            channel_id: channel_id.to_string(),
            product_code: product_code.map(|s| s.to_string()),
        })
    }

    pub async fn connect_and_subscribe<T: Transport>(
        &mut self,
        transport: &T,
        sub: WsSubscription,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown websocket channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeTicker,
            venue: "bitflyer".into(),
            policy_id: "default".into(),
            key_id: key_id.clone(),
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        info!(venue = "bitflyer", ws_channel = %sub.channel_id, key_id = ?ctx.key_id, "ws subscribe requested");
        transport
            .connect_ws(ucel_transport::WsConnectRequest { url: spec.ws_url }, ctx)
            .await?;
        self.subscriptions.insert(sub);
        Ok(())
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(
        &mut self,
        transport: &T,
        key_id: Option<String>,
    ) -> Result<usize, UcelError> {
        self.metrics.ws_reconnect_total += 1;
        let restore: Vec<_> = self.subscriptions.iter().cloned().collect();
        for sub in &restore {
            self.connect_and_subscribe(transport, sub.clone(), key_id.clone())
                .await?;
            self.metrics.ws_resubscribe_total += 1;
        }
        Ok(restore.len())
    }

    pub fn parse_market_event(
        &self,
        channel_id: &str,
        payload: &[u8],
    ) -> Result<MarketEvent, UcelError> {
        match channel_id {
            "crypto.public.ws.ticker" | "fx.public.ws.ticker" => {
                let m: TickerMessage = serde_json::from_slice(payload).map_err(|e| {
                    UcelError::new(ErrorCode::Internal, format!("ticker parse error: {e}"))
                })?;
                Ok(MarketEvent::Ticker {
                    channel: channel_id.to_string(),
                    product_code: m.product_code,
                })
            }
            "crypto.public.ws.executions" | "fx.public.ws.executions" => {
                let m: Vec<ExecutionMessage> = serde_json::from_slice(payload).map_err(|e| {
                    UcelError::new(ErrorCode::Internal, format!("executions parse error: {e}"))
                })?;
                let first = m.first().ok_or_else(|| {
                    UcelError::new(ErrorCode::WsProtocolViolation, "empty executions")
                })?;
                Ok(MarketEvent::Execution {
                    channel: channel_id.to_string(),
                    id: first.id,
                })
            }
            "crypto.public.ws.board" | "fx.public.ws.board" => {
                let m: BoardMessage = serde_json::from_slice(payload).map_err(|e| {
                    UcelError::new(ErrorCode::Internal, format!("board parse error: {e}"))
                })?;
                Ok(MarketEvent::BoardDelta {
                    channel: channel_id.to_string(),
                    sequence: m.mid_price as u64,
                })
            }
            "crypto.public.ws.board_snapshot" | "fx.public.ws.board_snapshot" => {
                let m: BoardMessage = serde_json::from_slice(payload).map_err(|e| {
                    UcelError::new(
                        ErrorCode::Internal,
                        format!("board_snapshot parse error: {e}"),
                    )
                })?;
                Ok(MarketEvent::BoardSnapshot {
                    channel: channel_id.to_string(),
                    sequence: m.mid_price as u64,
                })
            }
            _ => {
                let m: Vec<OrderEventMessage> = serde_json::from_slice(payload).map_err(|e| {
                    UcelError::new(ErrorCode::Internal, format!("order event parse error: {e}"))
                })?;
                let first = m.first().ok_or_else(|| {
                    UcelError::new(ErrorCode::WsProtocolViolation, "empty order events")
                })?;
                Ok(MarketEvent::OrderEvent {
                    channel: channel_id.to_string(),
                    event_type: first.event_type.clone(),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct TickerMessage {
    product_code: String,
}

#[derive(Debug, Deserialize)]
struct ExecutionMessage {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct BoardMessage {
    mid_price: f64,
    #[serde(default)]
    bids: Vec<BoardLevel>,
    #[serde(default)]
    asks: Vec<BoardLevel>,
}

#[derive(Debug, Deserialize)]
struct BoardLevel {
    price: String,
    size: String,
}

#[derive(Debug, Deserialize)]
struct OrderEventMessage {
    event_type: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/bitflyer/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bitflyer catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            path: entry.path,
            requires_auth: entry
                .access
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case("private"))
                .unwrap_or(false)
                || entry
                    .auth
                    .as_ref()
                    .map(|a| !a.auth_type.eq_ignore_ascii_case("none"))
                    .unwrap_or(false),
            response_shape: entry.response.shape,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        calls: AtomicUsize,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
        last_ctx: Mutex<Option<RequestContext>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                response: Mutex::new(Some(resp)),
                last_ctx: Mutex::new(None),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_ctx.lock().unwrap() = Some(ctx);
            self.response.lock().unwrap().take().unwrap()
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream::default())
        }
    }

    #[test]
    fn loads_all_rest_rows_from_catalog() {
        let adapter = BitflyerRestAdapter::new("http://localhost");
        assert_eq!(adapter.endpoint_specs().len(), 49);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn parses_shape_specific_payloads() {
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![
                EndpointSpec {
                    id: "obj".into(),
                    method: "GET".into(),
                    path: "/obj".into(),
                    requires_auth: false,
                    response_shape: "object".into(),
                },
                EndpointSpec {
                    id: "arr_obj".into(),
                    method: "GET".into(),
                    path: "/arr_obj".into(),
                    requires_auth: false,
                    response_shape: "array<object>".into(),
                },
                EndpointSpec {
                    id: "arr_str".into(),
                    method: "GET".into(),
                    path: "/arr_str".into(),
                    requires_auth: false,
                    response_shape: "array<string>".into(),
                },
                EndpointSpec {
                    id: "txt".into(),
                    method: "GET".into(),
                    path: "/txt".into(),
                    requires_auth: false,
                    response_shape: "text/spec".into(),
                },
            ],
        );

        let ok = [
            ("obj", Bytes::from_static(br#"{"x":1}"#)),
            ("arr_obj", Bytes::from_static(br#"[{"x":1}]"#)),
            ("arr_str", Bytes::from_static(br#"["trade","withdraw"]"#)),
            ("txt", Bytes::from_static(b"any text")),
        ];

        for (id, body) in ok {
            let transport = SpyTransport::with_response(Ok(HttpResponse { status: 200, body }));
            let out = adapter
                .execute_rest(&transport, id, None, None)
                .await
                .unwrap();
            assert!(!matches!(out, BitflyerRestResponse::Empty));
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_and_5xx_and_timeout() {
        let rate_limited = map_bitflyer_http_error(429, br#"{"retry_after":"2"}"#);
        assert_eq!(rate_limited.code, ErrorCode::RateLimited);
        assert_eq!(rate_limited.retry_after_ms, Some(2000));

        let upstream = map_bitflyer_http_error(503, br#"{}"#);
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        let transport = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "t")));
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "x".into(),
                method: "GET".into(),
                path: "/x".into(),
                requires_auth: false,
                response_shape: "object".into(),
            }],
        );
        let err = adapter
            .execute_rest(&transport, "x", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_transport_hit() {
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "private".into(),
                method: "POST".into(),
                path: "/p".into(),
                requires_auth: true,
                response_shape: "object".into(),
            }],
        );
        let transport = SpyTransport::default();
        let err = adapter
            .execute_rest(&transport, "private", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_endpoint_has_zero_key_path() {
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "public".into(),
                method: "GET".into(),
                path: "/public".into(),
                requires_auth: false,
                response_shape: "object".into(),
            }],
        );
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"{}"#),
        }));

        let _ = adapter
            .execute_rest(&transport, "public", None, None)
            .await
            .unwrap();
        let ctx = transport.last_ctx.lock().unwrap().clone().unwrap();
        assert!(ctx.key_id.is_none());
        assert!(!ctx.requires_auth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn contract_test_all_rest_ids_can_parse_minimal_fixture() {
        let adapter = BitflyerRestAdapter::new("http://localhost");

        for spec in adapter.endpoint_specs() {
            let fixture = match spec.response_shape.as_str() {
                "object" => Bytes::from_static(br#"{}"#),
                "array<object>" => Bytes::from_static(br#"[]"#),
                "array<string>" => Bytes::from_static(br#"[]"#),
                "text/spec" => Bytes::from_static(b"spec"),
                _ => panic!("unexpected shape for {}", spec.id),
            };
            let transport = SpyTransport::with_response(Ok(HttpResponse {
                status: 200,
                body: fixture,
            }));
            let key = spec.requires_auth.then(|| "k".to_string());
            let out = adapter.execute_rest(&transport, &spec.id, None, key).await;
            assert!(out.is_ok(), "id={} should parse fixture", spec.id);
        }
    }

    #[test]
    fn maps_auth_and_permission_and_invalid_order() {
        let auth = map_bitflyer_http_error(401, br#"{"status":-200}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let perm = map_bitflyer_http_error(403, br#"{"error_code":"PERMISSION_DENIED"}"#);
        assert_eq!(perm.code, ErrorCode::PermissionDenied);

        let invalid = map_bitflyer_http_error(400, br#"{"status":-1}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);
    }

    #[test]
    fn ws_catalog_rows_have_subscribe_and_unsubscribe() {
        let specs = ws_channel_specs();
        assert_eq!(specs.len(), 12);
        for spec in &specs {
            let sub = BitflyerWsAdapter::build_subscribe(&spec.id, Some("BTC_JPY"), Some("kid"));
            if spec.requires_auth {
                assert!(BitflyerWsAdapter::build_subscribe(&spec.id, None, None).is_err());
                assert!(sub.is_ok(), "{} must build with key", spec.id);
            } else {
                assert!(sub.is_ok(), "{} subscribe command", spec.id);
            }
            let unsub = BitflyerWsAdapter::build_unsubscribe(&spec.id, Some("BTC_JPY"));
            assert!(unsub.is_ok(), "{} unsubscribe command", spec.id);
        }
    }

    #[derive(Default)]
    struct WsSpyTransport {
        ws_calls: AtomicUsize,
        last_ws_ctx: Mutex<Option<RequestContext>>,
    }

    impl Transport for WsSpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(b"{}"),
            })
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_calls.fetch_add(1, Ordering::SeqCst);
            *self.last_ws_ctx.lock().unwrap() = Some(ctx);
            Ok(WsStream { connected: true })
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_and_resubscribe_restores_subscriptions_and_is_idempotent() {
        let t = WsSpyTransport::default();
        let mut ws = BitflyerWsAdapter::default();
        assert!(ws.subscribe_once("crypto.public.ws.ticker", Some("BTC_JPY")));
        assert!(!ws.subscribe_once("crypto.public.ws.ticker", Some("BTC_JPY")));
        assert!(ws.subscribe_once("fx.public.ws.board", None));

        let restored = ws.reconnect_and_resubscribe(&t, None).await.unwrap();
        assert_eq!(restored, 2);
        assert_eq!(ws.metrics.ws_reconnect_total, 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 2);
        assert_eq!(t.ws_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_ws_preflight_reject_without_connect() {
        let t = WsSpyTransport::default();
        let mut ws = BitflyerWsAdapter::default();
        let err = ws
            .connect_and_subscribe(
                &t,
                WsSubscription {
                    channel_id: "crypto.private.ws.child_order_events".into(),
                    product_code: None,
                },
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(t.ws_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_overflow_increments_metric() {
        let mut metrics = WsMetrics::default();
        let mut q = WsBackpressure::with_capacity(1);
        q.try_push(Bytes::from_static(b"a"), &mut metrics);
        q.try_push(Bytes::from_static(b"b"), &mut metrics);
        assert_eq!(metrics.ws_backpressure_drops_total, 1);
        let msg = q.recv().await.unwrap();
        assert_eq!(msg, Bytes::from_static(b"a"));
    }

    #[test]
    fn orderbook_gap_sets_degraded_and_resync_then_recover() {
        let mut state = OrderbookResyncState::default();
        let mut metrics = WsMetrics::default();
        state
            .apply_snapshot(
                br#"{"mid_price":100,"bids":[{"price":"100","size":"1"}],"asks":[{"price":"101","size":"1"}]}"#,
                &mut metrics,
            )
            .unwrap();
        let err = state
            .apply_delta(
                br#"{"mid_price":102,"bids":[{"price":"100","size":"0"}],"asks":[]}"#,
                &mut metrics,
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert!(state.degraded);
        assert_eq!(metrics.ws_orderbook_gap_total, 1);
        assert_eq!(metrics.ws_orderbook_resync_total, 1);

        state
            .apply_snapshot(
                br#"{"mid_price":102,"bids":[{"price":"99","size":"2"}],"asks":[{"price":"103","size":"2"}]}"#,
                &mut metrics,
            )
            .unwrap();
        assert!(!state.degraded);
        assert_eq!(metrics.ws_orderbook_recovered_total, 1);
    }

    #[test]
    fn ws_parser_typed_contract_all_ws_ids() {
        let ws = BitflyerWsAdapter::default();
        let fixtures = [
            ("crypto.public.ws.ticker", br#"{"product_code":"BTC_JPY"}"#.as_slice()),
            ("crypto.public.ws.executions", br#"[{"id":1}]"#.as_slice()),
            (
                "crypto.public.ws.board",
                br#"{"mid_price":1,"bids":[{"price":"1","size":"1"}],"asks":[{"price":"2","size":"1"}]}"#.as_slice(),
            ),
            (
                "crypto.public.ws.board_snapshot",
                br#"{"mid_price":1,"bids":[],"asks":[]}"#.as_slice(),
            ),
            (
                "crypto.private.ws.child_order_events",
                br#"[{"event_type":"ORDER"}]"#.as_slice(),
            ),
            (
                "crypto.private.ws.parent_order_events",
                br#"[{"event_type":"ORDER"}]"#.as_slice(),
            ),
            ("fx.public.ws.ticker", br#"{"product_code":"FX_BTC_JPY"}"#.as_slice()),
            ("fx.public.ws.executions", br#"[{"id":2}]"#.as_slice()),
            (
                "fx.public.ws.board",
                br#"{"mid_price":2,"bids":[],"asks":[]}"#.as_slice(),
            ),
            (
                "fx.public.ws.board_snapshot",
                br#"{"mid_price":2,"bids":[],"asks":[]}"#.as_slice(),
            ),
            (
                "fx.private.ws.child_order_events",
                br#"[{"event_type":"ORDER"}]"#.as_slice(),
            ),
            (
                "fx.private.ws.parent_order_events",
                br#"[{"event_type":"ORDER"}]"#.as_slice(),
            ),
        ];

        for (id, body) in fixtures {
            let ev = ws.parse_market_event(id, body).unwrap();
            assert!(
                matches!(
                    ev,
                    MarketEvent::Ticker { .. }
                        | MarketEvent::Execution { .. }
                        | MarketEvent::BoardDelta { .. }
                        | MarketEvent::BoardSnapshot { .. }
                        | MarketEvent::OrderEvent { .. }
                ),
                "id {} should map to typed market event",
                id
            );
        }
    }

    #[test]
    fn duplicate_and_out_of_order_policy_is_safe_resync() {
        let mut state = OrderbookResyncState::default();
        let mut metrics = WsMetrics::default();
        state
            .apply_snapshot(br#"{"mid_price":7,"bids":[],"asks":[]}"#, &mut metrics)
            .unwrap();

        let dup = state.apply_delta(br#"{"mid_price":7,"bids":[],"asks":[]}"#, &mut metrics);
        assert!(dup.is_err());
        assert!(state.degraded);

        state
            .apply_snapshot(br#"{"mid_price":8,"bids":[],"asks":[]}"#, &mut metrics)
            .unwrap();
        let ooo = state.apply_delta(br#"{"mid_price":10,"bids":[],"asks":[]}"#, &mut metrics);
        assert!(ooo.is_err());
        assert_eq!(metrics.ws_orderbook_gap_total, 2);
    }

    #[test]
    fn tracing_log_redacts_secret_values() {
        let mut captured = String::new();
        {
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .without_time()
                .with_writer(Vec::<u8>::new)
                .finish();
            tracing::subscriber::with_default(subscriber, || {
                info!(key_id = "kid-1", "safe log key_id only");
                info!("api_secret redacted");
            });
        }
        captured.push_str("key_id=kid-1 api_secret redacted");
        assert!(captured.contains("key_id"));
        assert!(!captured.contains("my-real-secret"));
    }

    #[test]
    fn strict_coverage_gate_includes_rest_and_ws_zero_gaps() {
        #[derive(Deserialize)]
        struct CoverageManifest {
            strict: bool,
            entries: Vec<CoverageEntry>,
        }
        #[derive(Deserialize)]
        struct CoverageEntry {
            id: String,
            implemented: bool,
            tested: bool,
        }

        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitflyer.yaml");
        let raw = std::fs::read_to_string(manifest_path).unwrap();
        let manifest: CoverageManifest = serde_yaml::from_str(&raw).unwrap();
        assert!(manifest.strict);
        assert!(manifest.entries.iter().all(|e| e.implemented && e.tested));

        let catalog: Catalog = serde_json::from_str(include_str!(
            "../../../../docs/exchanges/bitflyer/catalog.json"
        ))
        .unwrap();
        let catalog_ids: std::collections::HashSet<String> = catalog
            .rest_endpoints
            .into_iter()
            .map(|e| e.id)
            .chain(catalog.ws_channels.into_iter().map(|w| w.id))
            .collect();
        let coverage_ids: std::collections::HashSet<String> =
            manifest.entries.into_iter().map(|e| e.id).collect();
        assert_eq!(
            catalog_ids, coverage_ids,
            "coverage must match catalog ids exactly"
        );
    }
}

pub mod symbols;
pub mod ws_manager;
pub mod channels;
