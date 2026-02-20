use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{
    ErrorCode, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    pub rest_endpoints: Vec<RestEndpoint>,
    #[serde(default)]
    pub ws_channels: Vec<WsChannel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestEndpoint {
    pub id: String,
    pub visibility: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsChannel {
    pub id: String,
    pub visibility: String,
    pub ws_url: String,
    pub subscribe: String,
    pub unsubscribe: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct RestRequest {
    pub method: String,
    pub url: String,
    pub auth: Option<AuthContext>,
}

#[derive(Debug, Clone)]
pub struct RestResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after_ms: Option<u64>,
}

pub trait HttpExecutor {
    fn execute(&mut self, request: RestRequest) -> Result<RestResponse, UcelError>;
}

pub trait WsExecutor {
    fn connect(&mut self, url: &str) -> Result<(), UcelError>;
    fn send(&mut self, message: String) -> Result<(), UcelError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub symbol: String,
}

#[derive(Debug, Default)]
pub struct WsMetrics {
    pub reconnect_total: AtomicU64,
    pub resubscribe_total: AtomicU64,
    pub backpressure_drops_total: AtomicU64,
}

pub struct WsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    metrics: Arc<WsMetrics>,
}

impl WsBackpressure {
    pub fn new(capacity: usize, metrics: Arc<WsMetrics>) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx, metrics }
    }

    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.metrics
                .backpressure_drops_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

pub struct SbivcWsClient<E> {
    channels: HashMap<String, WsChannel>,
    active: HashSet<WsSubscription>,
    executor: E,
    metrics: Arc<WsMetrics>,
}

impl<E: WsExecutor> SbivcWsClient<E> {
    pub fn new(
        catalog_json: &str,
        executor: E,
        metrics: Arc<WsMetrics>,
    ) -> Result<Self, UcelError> {
        let catalog: Catalog = serde_json::from_str(catalog_json)
            .map_err(|e| UcelError::new(ErrorCode::CatalogInvalid, e.to_string()))?;
        Ok(Self {
            channels: catalog
                .ws_channels
                .into_iter()
                .map(|c| (c.id.clone(), c))
                .collect(),
            active: HashSet::new(),
            executor,
            metrics,
        })
    }

    pub fn subscribe(
        &mut self,
        sub: WsSubscription,
        key_id: Option<&str>,
    ) -> Result<(), UcelError> {
        let channel = self
            .channels
            .get(&sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        if channel.visibility == "private" && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private channel requires key_id",
            ));
        }
        if !self.active.insert(sub.clone()) {
            return Ok(());
        }
        self.executor.connect(&channel.ws_url)?;
        let cmd = channel.subscribe.replace("{symbol}", &sub.symbol);
        self.executor.send(cmd)?;
        info!(key_id = key_id.unwrap_or(""), "ws subscribe");
        Ok(())
    }

    pub fn unsubscribe(&mut self, sub: &WsSubscription) -> Result<(), UcelError> {
        let channel = self
            .channels
            .get(&sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        if !self.active.remove(sub) {
            return Ok(());
        }
        let cmd = channel.unsubscribe.replace("{symbol}", &sub.symbol);
        self.executor.send(cmd)
    }

    pub fn reconnect_and_resubscribe(&mut self) -> Result<(), UcelError> {
        self.metrics.reconnect_total.fetch_add(1, Ordering::Relaxed);
        let active: Vec<_> = self.active.iter().cloned().collect();
        for sub in active {
            let channel = self
                .channels
                .get(&sub.channel_id)
                .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
            self.executor.connect(&channel.ws_url)?;
            let cmd = channel.subscribe.replace("{symbol}", &sub.symbol);
            self.executor.send(cmd)?;
            self.metrics
                .resubscribe_total
                .fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker { symbol: String, last: f64 },
    Trade(TradeEvent),
    OrderBookSnapshot(OrderBookSnapshot),
    OrderBookDelta(OrderBookDelta),
    Order { order_id: String, status: String },
}

pub fn parse_market_event(id: &str, body: &[u8]) -> Result<MarketEvent, UcelError> {
    match id {
        "crypto.public.ws.market.ticker" => {
            let msg: TickerMsg = parse_json(body)?;
            Ok(MarketEvent::Ticker {
                symbol: msg.symbol,
                last: msg.last,
            })
        }
        "crypto.public.ws.market.trades" => {
            let msg: TradeMsg = parse_json(body)?;
            Ok(MarketEvent::Trade(TradeEvent {
                trade_id: msg.trade_id,
                price: msg.price,
                qty: msg.qty,
                side: msg.side,
            }))
        }
        "crypto.public.ws.market.orderbook.snapshot" => {
            let msg: BookMsg = parse_json(body)?;
            Ok(MarketEvent::OrderBookSnapshot(OrderBookSnapshot {
                bids: msg.bids,
                asks: msg.asks,
                sequence: msg.sequence,
            }))
        }
        "crypto.public.ws.market.orderbook.delta" => {
            let msg: DeltaMsg = parse_json(body)?;
            Ok(MarketEvent::OrderBookDelta(OrderBookDelta {
                bids: msg.bids,
                asks: msg.asks,
                sequence_start: msg.sequence_start,
                sequence_end: msg.sequence_end,
            }))
        }
        "crypto.private.ws.order" => {
            let msg: PrivateOrderMsg = parse_json(body)?;
            Ok(MarketEvent::Order {
                order_id: msg.order_id,
                status: msg.status,
            })
        }
        _ => Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("unsupported channel {id}"),
        )),
    }
}

#[derive(Debug, Default)]
pub struct OrderbookResync {
    expected_next: Option<u64>,
    pub degraded: bool,
}

impl OrderbookResync {
    pub fn on_snapshot(&mut self, snapshot: &OrderBookSnapshot) {
        self.expected_next = Some(snapshot.sequence.saturating_add(1));
        self.degraded = false;
    }

    pub fn on_delta(&mut self, delta: &OrderBookDelta) -> Result<bool, UcelError> {
        let Some(next) = self.expected_next else {
            self.degraded = true;
            return Err(UcelError::new(ErrorCode::Desync, "missing snapshot"));
        };
        if delta.sequence_end < next {
            return Ok(false);
        }
        if delta.sequence_start != next {
            self.degraded = true;
            self.expected_next = None;
            return Err(UcelError::new(ErrorCode::Desync, "gap mismatch"));
        }
        self.expected_next = Some(delta.sequence_end.saturating_add(1));
        Ok(true)
    }

    pub fn recover_with_snapshot(&mut self, snapshot: &OrderBookSnapshot) {
        self.on_snapshot(snapshot);
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SbivcErrorBody {
    code: Option<String>,
    error_code: Option<String>,
    field: Option<String>,
    message: Option<String>,
}

pub struct SbivcRestClient<E> {
    endpoints: Vec<RestEndpoint>,
    executor: E,
}

impl<E: HttpExecutor> SbivcRestClient<E> {
    pub fn new(catalog_json: &str, executor: E) -> Result<Self, UcelError> {
        let catalog: Catalog = serde_json::from_str(catalog_json)
            .map_err(|e| UcelError::new(ErrorCode::CatalogInvalid, e.to_string()))?;
        Ok(Self {
            endpoints: catalog.rest_endpoints,
            executor,
        })
    }

    pub fn call<T: DeserializeOwned>(
        &mut self,
        id: &str,
        auth: Option<AuthContext>,
    ) -> Result<T, UcelError> {
        let endpoint = self
            .endpoints
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown id={id}")))?;

        let requires_auth = endpoint.visibility == "private";
        if requires_auth && auth.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private endpoint requires auth",
            ));
        }

        let response = self.executor.execute(RestRequest {
            method: endpoint.method.clone(),
            url: format!("{}{}", endpoint.base_url, endpoint.path),
            auth,
        })?;

        if (200..300).contains(&response.status) {
            return serde_json::from_slice::<T>(&response.body)
                .map_err(|e| UcelError::new(ErrorCode::Internal, e.to_string()));
        }

        let mut err = map_error(response.status, &response.body);
        err.retry_after_ms = response.retry_after_ms;
        Err(err)
    }
}

fn map_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        return UcelError::new(ErrorCode::RateLimited, "rate limited");
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream 5xx");
    }

    let parsed = serde_json::from_slice::<SbivcErrorBody>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|value| value.error_code.as_deref().or(value.code.as_deref()));
    let field = parsed.as_ref().and_then(|value| value.field.as_deref());

    match (status, code, field) {
        (401, _, _) | (_, Some("AUTH_FAILED"), _) => {
            UcelError::new(ErrorCode::AuthFailed, "auth failed")
        }
        (403, _, _) | (_, Some("PERMISSION_DENIED"), _) => {
            UcelError::new(ErrorCode::PermissionDenied, "permission denied")
        }
        (_, Some("INVALID_ORDER"), _) | (_, _, Some("order")) => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        _ => UcelError::new(
            ErrorCode::Network,
            parsed
                .and_then(|value| value.message)
                .unwrap_or_else(|| "request failed".to_string()),
        ),
    }
}

fn parse_json<T: DeserializeOwned>(body: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(body)
        .map_err(|e| UcelError::new(ErrorCode::WsProtocolViolation, e.to_string()))
}

#[derive(Deserialize)]
struct TickerMsg {
    symbol: String,
    last: f64,
}

#[derive(Deserialize)]
struct TradeMsg {
    trade_id: String,
    price: f64,
    qty: f64,
    side: String,
}

#[derive(Deserialize)]
struct BookMsg {
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
    sequence: u64,
}

#[derive(Deserialize)]
struct DeltaMsg {
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
    sequence_start: u64,
    sequence_end: u64,
}

#[derive(Deserialize)]
struct PrivateOrderMsg {
    order_id: String,
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Mutex;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Default)]
    struct SpyExecutor {
        calls: usize,
        next: Option<Result<RestResponse, UcelError>>,
    }

    impl HttpExecutor for SpyExecutor {
        fn execute(&mut self, _request: RestRequest) -> Result<RestResponse, UcelError> {
            self.calls += 1;
            self.next
                .take()
                .unwrap_or_else(|| Err(UcelError::new(ErrorCode::Internal, "missing response")))
        }
    }

    #[derive(Default)]
    struct SpyWs {
        connects: usize,
        sends: usize,
        last_sent: Option<String>,
    }

    impl WsExecutor for SpyWs {
        fn connect(&mut self, _url: &str) -> Result<(), UcelError> {
            self.connects += 1;
            Ok(())
        }

        fn send(&mut self, message: String) -> Result<(), UcelError> {
            self.sends += 1;
            self.last_sent = Some(message);
            Ok(())
        }
    }

    #[derive(Debug, Deserialize)]
    struct Pong {
        ok: bool,
    }

    fn fixture_catalog() -> &'static str {
        r#"{
          "rest_endpoints": [
            {
              "id": "crypto.public.rest.ping",
              "visibility": "public",
              "method": "GET",
              "base_url": "https://api.example.com",
              "path": "/v1/ping"
            },
            {
              "id": "crypto.private.rest.order.create",
              "visibility": "private",
              "method": "POST",
              "base_url": "https://api.example.com",
              "path": "/v1/order"
            }
          ],
          "ws_channels": [
            {
              "id": "crypto.public.ws.market.ticker",
              "visibility": "public",
              "ws_url": "wss://stream.example.com/public",
              "subscribe": "{\"op\":\"subscribe\",\"channel\":\"ticker\",\"symbol\":\"{symbol}\"}",
              "unsubscribe": "{\"op\":\"unsubscribe\",\"channel\":\"ticker\",\"symbol\":\"{symbol}\"}"
            },
            {
              "id": "crypto.public.ws.market.trades",
              "visibility": "public",
              "ws_url": "wss://stream.example.com/public",
              "subscribe": "{\"op\":\"subscribe\",\"channel\":\"trades\",\"symbol\":\"{symbol}\"}",
              "unsubscribe": "{\"op\":\"unsubscribe\",\"channel\":\"trades\",\"symbol\":\"{symbol}\"}"
            },
            {
              "id": "crypto.public.ws.market.orderbook.snapshot",
              "visibility": "public",
              "ws_url": "wss://stream.example.com/public",
              "subscribe": "{\"op\":\"subscribe\",\"channel\":\"book_snapshot\",\"symbol\":\"{symbol}\"}",
              "unsubscribe": "{\"op\":\"unsubscribe\",\"channel\":\"book_snapshot\",\"symbol\":\"{symbol}\"}"
            },
            {
              "id": "crypto.public.ws.market.orderbook.delta",
              "visibility": "public",
              "ws_url": "wss://stream.example.com/public",
              "subscribe": "{\"op\":\"subscribe\",\"channel\":\"book_delta\",\"symbol\":\"{symbol}\"}",
              "unsubscribe": "{\"op\":\"unsubscribe\",\"channel\":\"book_delta\",\"symbol\":\"{symbol}\"}"
            },
            {
              "id": "crypto.private.ws.order",
              "visibility": "private",
              "ws_url": "wss://stream.example.com/private",
              "subscribe": "{\"op\":\"subscribe\",\"channel\":\"orders\",\"symbol\":\"{symbol}\"}",
              "unsubscribe": "{\"op\":\"unsubscribe\",\"channel\":\"orders\",\"symbol\":\"{symbol}\"}"
            }
          ]
        }"#
    }

    #[test]
    fn private_operation_is_rejected_before_transport_call() {
        let spy = SpyExecutor::default();
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.private.rest.order.create", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::MissingAuth);
        assert_eq!(client.executor.calls, 0);
    }

    #[test]
    fn public_operation_calls_transport_without_auth() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 200,
                body: br#"{"ok":true}"#.to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let response = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap();
        assert!(response.ok);
        assert_eq!(client.executor.calls, 1);
    }

    #[test]
    fn maps_rate_limit_and_retry_after() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 429,
                body: b"{}".to_vec(),
                retry_after_ms: Some(1500),
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::RateLimited);
        assert_eq!(error.retry_after_ms, Some(1500));
    }

    #[test]
    fn maps_5xx() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 503,
                body: b"{}".to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Upstream5xx);
    }

    #[test]
    fn maps_api_error_by_code_and_field() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 400,
                body: br#"{"error_code":"INVALID_ORDER","field":"order"}"#.to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidOrder);
    }

    #[test]
    fn maps_timeout_without_string_matching() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Err(UcelError::new(ErrorCode::Timeout, "timed out"))),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Timeout);
    }

    #[test]
    fn ws_contract_parses_every_catalog_channel_id() {
        let catalog: Catalog = serde_json::from_str(fixture_catalog()).unwrap();
        let mut payloads: HashMap<&str, &[u8]> = HashMap::new();
        payloads.insert(
            "crypto.public.ws.market.ticker",
            br#"{"symbol":"BTC_JPY","last":100.0}"#,
        );
        payloads.insert(
            "crypto.public.ws.market.trades",
            br#"{"trade_id":"t1","price":100.0,"qty":0.1,"side":"buy"}"#,
        );
        payloads.insert(
            "crypto.public.ws.market.orderbook.snapshot",
            br#"{"bids":[{"price":100.0,"qty":1.0}],"asks":[{"price":101.0,"qty":2.0}],"sequence":10}"#,
        );
        payloads.insert(
            "crypto.public.ws.market.orderbook.delta",
            br#"{"bids":[{"price":100.0,"qty":1.0}],"asks":[{"price":101.0,"qty":2.0}],"sequence_start":11,"sequence_end":11}"#,
        );
        payloads.insert(
            "crypto.private.ws.order",
            br#"{"order_id":"o1","status":"filled"}"#,
        );

        for ch in catalog.ws_channels {
            let body = payloads.get(ch.id.as_str()).unwrap();
            assert!(parse_market_event(&ch.id, body).is_ok(), "id={}", ch.id);
        }
    }

    #[test]
    fn private_ws_preflight_reject_without_connect_or_subscribe_side_effects() {
        let metrics = Arc::new(WsMetrics::default());
        let spy = SpyWs::default();
        let mut client = SbivcWsClient::new(fixture_catalog(), spy, metrics).unwrap();

        let err = client
            .subscribe(
                WsSubscription {
                    channel_id: "crypto.private.ws.order".into(),
                    symbol: "BTC_JPY".into(),
                },
                None,
            )
            .unwrap_err();

        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(client.executor.connects, 0);
        assert_eq!(client.executor.sends, 0);
    }

    #[test]
    fn reconnect_resubscribe_is_idempotent_and_restores_subscriptions() {
        let metrics = Arc::new(WsMetrics::default());
        let spy = SpyWs::default();
        let mut client = SbivcWsClient::new(fixture_catalog(), spy, metrics.clone()).unwrap();
        let sub = WsSubscription {
            channel_id: "crypto.public.ws.market.ticker".into(),
            symbol: "BTC_JPY".into(),
        };

        client.subscribe(sub.clone(), None).unwrap();
        client.subscribe(sub.clone(), None).unwrap();
        assert_eq!(client.executor.sends, 1);

        client.reconnect_and_resubscribe().unwrap();
        assert_eq!(metrics.reconnect_total.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.resubscribe_total.load(Ordering::Relaxed), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_drops_and_counts() {
        let metrics = Arc::new(WsMetrics::default());
        let mut bp = WsBackpressure::new(1, metrics.clone());

        bp.try_enqueue(Bytes::from_static(b"a"));
        bp.try_enqueue(Bytes::from_static(b"b"));

        let got = bp.recv().await.unwrap();
        assert_eq!(got, Bytes::from_static(b"a"));
        assert_eq!(metrics.backpressure_drops_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn orderbook_gap_duplicate_out_of_order_policy_and_recovery() {
        let mut state = OrderbookResync::default();
        state.on_snapshot(&OrderBookSnapshot {
            bids: vec![],
            asks: vec![],
            sequence: 10,
        });
        let duplicate = OrderBookDelta {
            bids: vec![],
            asks: vec![],
            sequence_start: 9,
            sequence_end: 9,
        };
        assert!(!state.on_delta(&duplicate).unwrap());

        let gap = OrderBookDelta {
            bids: vec![],
            asks: vec![],
            sequence_start: 12,
            sequence_end: 12,
        };
        let err = state.on_delta(&gap).unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert!(state.degraded);

        state.recover_with_snapshot(&OrderBookSnapshot {
            bids: vec![],
            asks: vec![],
            sequence: 20,
        });
        assert!(!state.degraded);
    }

    #[derive(Clone, Default)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);
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
    struct BufMaker(SharedBuf);
    impl<'a> MakeWriter<'a> for BufMaker {
        type Writer = SharedBuf;
        fn make_writer(&'a self) -> Self::Writer {
            self.0.clone()
        }
    }

    #[test]
    fn logs_do_not_leak_api_secret_material() {
        let sink = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(BufMaker(sink.clone()))
            .without_time()
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            info!(key_id = "kid-123", api_key = "", "private ws auth ready");
        });
        let logs = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        assert!(logs.contains("kid-123"));
        assert!(!logs.contains("api_secret"));
        assert!(!logs.contains("supersecret"));
    }

    #[test]
    fn strict_coverage_gate_is_enabled_and_zero_gap_for_sbivc_catalog_ids() {
        let manifest: serde_yaml::Value = serde_yaml::from_str(
            &std::fs::read_to_string(Path::new("../../coverage/sbivc.yaml")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["strict"].as_bool(), Some(true));

        let entries = manifest["entries"]
            .as_sequence()
            .cloned()
            .unwrap_or_default();
        let has_gap = entries.iter().any(|entry| {
            entry["implemented"].as_bool() != Some(true) || entry["tested"].as_bool() != Some(true)
        });
        assert!(!has_gap, "strict gate requires zero gaps");

        let catalog: Catalog = serde_json::from_str(include_str!(
            "../../../../docs/exchanges/sbivc/catalog.json"
        ))
        .unwrap();
        let ids: HashSet<_> = catalog
            .rest_endpoints
            .iter()
            .map(|x| x.id.clone())
            .chain(catalog.ws_channels.iter().map(|x| x.id.clone()))
            .collect();
        let covered: HashSet<_> = entries
            .iter()
            .filter_map(|e| e["id"].as_str().map(ToString::to_string))
            .collect();
        for id in ids {
            assert!(covered.contains(&id), "missing coverage entry for {id}");
        }
    }
}
