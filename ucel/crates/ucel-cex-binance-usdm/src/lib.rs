use bytes::Bytes;
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{
    enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport, WsConnectRequest,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub channel: &'static str,
    pub requires_auth: bool,
}

pub const WS_CHANNELS: [WsChannelSpec; 10] = [
    WsChannelSpec {
        id: "usdm.public.ws.market.root",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "!markPrice@arr",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.aggtrade",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@aggTrade",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.markprice",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@markPrice",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.kline",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@kline_<interval>",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.bookticker",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@bookTicker",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.liquidation",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@forceOrder",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.depth.partial",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@depth<levels>",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.depth.diff",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@depth",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.wsapi.general",
        ws_url: "wss://ws-fapi.binance.com/ws-fapi/v1",
        channel: "rpc",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.private.ws.userdata.events",
        ws_url: "wss://fstream.binance.com/ws/<listenKey>",
        channel: "userdata",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 6] = [
    EndpointSpec {
        id: "usdm.public.rest.general.ref",
        method: "GET",
        base_url: "docs://binance-usdm",
        path: "/general-info",
        requires_auth: false,
    },
    EndpointSpec {
        id: "usdm.public.rest.errors.ref",
        method: "GET",
        base_url: "docs://binance-usdm",
        path: "/error-code",
        requires_auth: false,
    },
    EndpointSpec {
        id: "usdm.public.rest.market.ref",
        method: "GET",
        base_url: "docs://binance-usdm",
        path: "/market-data/rest-api",
        requires_auth: false,
    },
    EndpointSpec {
        id: "usdm.private.rest.trade.ref",
        method: "POST",
        base_url: "docs://binance-usdm",
        path: "/trade/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "usdm.private.rest.account.ref",
        method: "GET",
        base_url: "docs://binance-usdm",
        path: "/account/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "usdm.private.rest.listenkey.ref",
        method: "POST",
        base_url: "docs://binance-usdm",
        path: "/user-data-streams/rest-api",
        requires_auth: true,
    },
];

#[derive(Debug, Clone, Default)]
pub struct WsMetrics {
    pub reconnect_total: u64,
    pub resubscribe_total: u64,
    pub ws_drop_total: u64,
    pub ws_orderbook_gap_total: u64,
}

#[derive(Debug)]
pub struct WsSession {
    subscribed: HashSet<String>,
    pub metrics: WsMetrics,
    tx: mpsc::Sender<Bytes>,
    pub dropped_messages: u64,
}

impl WsSession {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<Bytes>) {
        let (tx, rx) = mpsc::channel(capacity);
        (
            Self {
                subscribed: HashSet::new(),
                metrics: WsMetrics::default(),
                tx,
                dropped_messages: 0,
            },
            rx,
        )
    }

    pub fn subscribe(&mut self, channel: &str) {
        self.subscribed.insert(channel.to_string());
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.subscribed.remove(channel);
    }

    pub fn reconnect_and_resubscribe(&mut self) -> Vec<String> {
        self.metrics.reconnect_total += 1;
        self.metrics.resubscribe_total += self.subscribed.len() as u64;
        let mut channels: Vec<_> = self.subscribed.iter().cloned().collect();
        channels.sort();
        channels
    }

    pub fn push_event(&mut self, payload: Bytes) {
        if self.tx.try_send(payload).is_err() {
            self.dropped_messages += 1;
            self.metrics.ws_drop_total += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "e")]
pub enum UsdmWsEvent {
    #[serde(rename = "aggTrade")]
    AggTrade { s: String, p: String, q: String },
    #[serde(rename = "markPriceUpdate")]
    MarkPrice { s: String, p: String },
    #[serde(rename = "kline")]
    Kline { s: String, k: KlinePayload },
    #[serde(rename = "bookTicker")]
    BookTicker { s: String, b: String, a: String },
    #[serde(rename = "forceOrder")]
    Liquidation { o: LiquidationOrder },
    #[serde(rename = "depthUpdate")]
    DepthDiff {
        s: String,
        U: u64,
        u: u64,
        b: Vec<[String; 2]>,
        a: Vec<[String; 2]>,
    },
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    UserOrderUpdate { i: String },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KlinePayload {
    pub i: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LiquidationOrder {
    pub s: String,
}

#[derive(Debug, Clone)]
pub struct OrderBookSync {
    pub last_update_id: u64,
    pub bids: BTreeMap<String, String>,
    pub asks: BTreeMap<String, String>,
    pub degraded: bool,
}

impl Default for OrderBookSync {
    fn default() -> Self {
        Self {
            last_update_id: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            degraded: false,
        }
    }
}

impl OrderBookSync {
    pub fn apply_snapshot(
        &mut self,
        last_update_id: u64,
        bids: Vec<[String; 2]>,
        asks: Vec<[String; 2]>,
    ) {
        self.last_update_id = last_update_id;
        self.bids = bids
            .into_iter()
            .map(|x| (x[0].clone(), x[1].clone()))
            .collect();
        self.asks = asks
            .into_iter()
            .map(|x| (x[0].clone(), x[1].clone()))
            .collect();
        self.degraded = false;
    }

    pub fn apply_diff(
        &mut self,
        first_id: u64,
        final_id: u64,
        bids: Vec<[String; 2]>,
        asks: Vec<[String; 2]>,
        metrics: &mut WsMetrics,
    ) {
        if self.last_update_id == 0
            || first_id > self.last_update_id + 1
            || final_id < self.last_update_id
        {
            self.degraded = true;
            metrics.ws_orderbook_gap_total += 1;
            return;
        }
        self.last_update_id = final_id;
        for level in bids {
            self.bids.insert(level[0].clone(), level[1].clone());
        }
        for level in asks {
            self.asks.insert(level[0].clone(), level[1].clone());
        }
    }

    pub fn resync(&mut self, snapshot_last_update_id: u64) {
        self.last_update_id = snapshot_last_update_id;
        self.degraded = false;
    }
}

pub fn preflight_ws_connect(
    channel_id: &str,
    key_id: Option<String>,
) -> Result<RequestContext, UcelError> {
    let spec = WS_CHANNELS
        .iter()
        .find(|s| s.id == channel_id)
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
    let ctx = RequestContext {
        trace_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        run_id: uuid::Uuid::new_v4().to_string(),
        op: if spec.id.contains("depth") {
            OpName::SubscribeOrderbook
        } else {
            OpName::FetchStatus
        },
        venue: "binance-usdm".into(),
        policy_id: "default".into(),
        key_id: if spec.requires_auth { key_id } else { None },
        requires_auth: spec.requires_auth,
    };
    enforce_auth_boundary(&ctx)?;
    Ok(ctx)
}

pub fn build_connect_request(channel_id: &str) -> Result<WsConnectRequest, UcelError> {
    let spec = WS_CHANNELS
        .iter()
        .find(|s| s.id == channel_id)
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
    info!(channel = %spec.id, "ws connect");
    Ok(WsConnectRequest {
        url: spec.ws_url.to_string(),
    })
}

#[derive(Debug, Clone)]
pub enum BinanceUsdmRestResponse {
    GeneralRef(GeneralRefResponse),
    ErrorsRef(ErrorsRefResponse),
    MarketRef(MarketRefResponse),
    TradeRef(TradeRefResponse),
    AccountRef(AccountRefResponse),
    ListenKeyRef(ListenKeyRefResponse),
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneralRefResponse {
    pub base_rules: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorsRefResponse {
    pub error_codes: Vec<BinanceErrorCodeRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceErrorCodeRef {
    pub code: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketRefResponse {
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeRefResponse {
    pub order_lifecycle_payloads: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountRefResponse {
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListenKeyRefResponse {
    pub lifecycle: String,
}

#[derive(Clone)]
pub struct BinanceUsdmRestAdapter {
    docs_base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BinanceUsdmRestAdapter {
    pub fn new(docs_base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            docs_base_url: Arc::from(docs_base_url.into()),
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
    ) -> Result<BinanceUsdmRestResponse, UcelError> {
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
            op: op_for_endpoint(endpoint_id),
            venue: "binance-usdm".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let base = if spec.base_url.starts_with("docs://") {
            self.docs_base_url.as_ref()
        } else {
            spec.base_url
        };
        let req = HttpRequest {
            method: spec.method.to_string(),
            path: format!("{base}{}", spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_binance_usdm_http_error(response.status, &response.body));
        }

        let parsed = match endpoint_id {
            "usdm.public.rest.general.ref" => {
                BinanceUsdmRestResponse::GeneralRef(parse_json(&response.body)?)
            }
            "usdm.public.rest.errors.ref" => {
                BinanceUsdmRestResponse::ErrorsRef(parse_json(&response.body)?)
            }
            "usdm.public.rest.market.ref" => {
                BinanceUsdmRestResponse::MarketRef(parse_json(&response.body)?)
            }
            "usdm.private.rest.trade.ref" => {
                BinanceUsdmRestResponse::TradeRef(parse_json(&response.body)?)
            }
            "usdm.private.rest.account.ref" => {
                BinanceUsdmRestResponse::AccountRef(parse_json(&response.body)?)
            }
            "usdm.private.rest.listenkey.ref" => {
                BinanceUsdmRestResponse::ListenKeyRef(parse_json(&response.body)?)
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

fn op_for_endpoint(endpoint_id: &str) -> OpName {
    match endpoint_id {
        "usdm.private.rest.trade.ref" => OpName::PlaceOrder,
        "usdm.private.rest.account.ref" => OpName::FetchBalances,
        "usdm.private.rest.listenkey.ref" => OpName::CreateWsAuthToken,
        _ => OpName::FetchStatus,
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct BinanceErrorEnvelope {
    code: i64,
}

pub fn map_binance_usdm_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = parse_retry_after_ms(body);
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let code = serde_json::from_slice::<BinanceErrorEnvelope>(body)
        .map(|env| env.code)
        .unwrap_or_default();

    match code {
        -1003 | -1008 | -1015 => {
            let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
            err.ban_risk = true;
            err.retry_after_ms = parse_retry_after_ms(body);
            err
        }
        -1002 | -1022 | -2014 | -2015 => {
            UcelError::new(ErrorCode::AuthFailed, "authentication failed")
        }
        -2017 => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        -1013 | -1100 | -1101 | -1102 | -1111 | -1116 | -1121 | -2010 | -2011 | -2019 => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        _ => UcelError::new(ErrorCode::Internal, "unmapped binance-usdm error"),
    }
}

fn parse_retry_after_ms(body: &[u8]) -> Option<u64> {
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(ms) = v.get("retryAfterMs").and_then(|x| x.as_u64()) {
            return Some(ms);
        }
    }

    std::str::from_utf8(body)
        .ok()
        .and_then(|b| b.split("retry_after_ms=").nth(1))
        .and_then(|s| s.trim().parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};

    #[test]
    fn ws_contract_covers_all_catalog_ids() {
        let ids: HashSet<&str> = WS_CHANNELS.iter().map(|s| s.id).collect();
        for expected in [
            "usdm.public.ws.market.root",
            "usdm.public.ws.market.aggtrade",
            "usdm.public.ws.market.markprice",
            "usdm.public.ws.market.kline",
            "usdm.public.ws.market.bookticker",
            "usdm.public.ws.market.liquidation",
            "usdm.public.ws.market.depth.partial",
            "usdm.public.ws.market.depth.diff",
            "usdm.public.ws.wsapi.general",
            "usdm.private.ws.userdata.events",
        ] {
            assert!(ids.contains(expected));
        }
    }

    #[test]
    fn private_preflight_rejects_without_auth() {
        let err = preflight_ws_connect("usdm.private.ws.userdata.events", None).unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[test]
    fn reconnect_and_resubscribe_is_idempotent() {
        let (mut session, _rx) = WsSession::new(2);
        session.subscribe("a");
        session.subscribe("a");
        session.subscribe("b");
        let set = session.reconnect_and_resubscribe();
        assert_eq!(set, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(session.metrics.resubscribe_total, 2);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_drops_on_overflow() {
        let (mut session, mut rx) = WsSession::new(1);
        session.push_event(Bytes::from_static(b"1"));
        session.push_event(Bytes::from_static(b"2"));
        assert_eq!(session.dropped_messages, 1);
        assert_eq!(rx.recv().await.unwrap(), Bytes::from_static(b"1"));
    }

    #[test]
    fn orderbook_gap_degraded_resync_recovered() {
        let mut ob = OrderBookSync::default();
        let mut m = WsMetrics::default();
        ob.apply_snapshot(100, vec![["1".into(), "1".into()]], vec![]);
        ob.apply_diff(105, 106, vec![], vec![], &mut m);
        assert!(ob.degraded);
        assert_eq!(m.ws_orderbook_gap_total, 1);
        ob.resync(200);
        assert!(!ob.degraded);
    }

    #[test]
    fn typed_deserialize_no_value() {
        let event: UsdmWsEvent =
            serde_json::from_str(r#"{"e":"aggTrade","s":"BTCUSDT","p":"100","q":"1"}"#).unwrap();
        match event {
            UsdmWsEvent::AggTrade { s, .. } => assert_eq!(s, "BTCUSDT"),
            _ => panic!(),
        }
    }

    #[test]
    fn strict_coverage_gate_for_binance_usdm_has_no_gaps() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../coverage/binance-usdm.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(manifest.strict);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }

    use serde::Deserialize;
    use std::collections::VecDeque;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Clone, Default)]
    struct SpyTransport {
        calls: Arc<AtomicUsize>,
        queue: Arc<Mutex<VecDeque<Result<HttpResponse, UcelError>>>>,
    }

    impl SpyTransport {
        fn enqueue_response(&self, status: u16, body: &'static str) {
            self.queue.lock().unwrap().push_back(Ok(HttpResponse {
                status,
                body: Bytes::from(body),
            }));
        }

        fn enqueue_error(&self, err: UcelError) {
            self.queue.lock().unwrap().push_back(Err(err));
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }

    impl ucel_transport::Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.queue
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err(UcelError::new(ErrorCode::Internal, "empty response queue")))
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream::default())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn catalog_contract_rest_all_rows_are_implemented_and_typed() {
        #[derive(Deserialize)]
        struct CatalogFixture {
            rest_endpoints: Vec<RestRow>,
        }
        #[derive(Deserialize)]
        struct RestRow {
            id: String,
        }

        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let raw =
            std::fs::read_to_string(repo_root.join("docs/exchanges/binance-usdm/catalog.json"))
                .unwrap();
        let catalog: CatalogFixture = serde_json::from_str(&raw).unwrap();

        let fixtures: [(&str, &str); 6] = [
            (
                "usdm.public.rest.general.ref",
                r#"{"base_rules":{"timing":true}}"#,
            ),
            (
                "usdm.public.rest.errors.ref",
                r#"{"error_codes":[{"code":-2015,"name":"REJECTED_MBX_KEY"}]}"#,
            ),
            (
                "usdm.public.rest.market.ref",
                r#"{"families":["exchangeInfo","depth","trades"]}"#,
            ),
            (
                "usdm.private.rest.trade.ref",
                r#"{"order_lifecycle_payloads":["newOrder","cancelOrder"]}"#,
            ),
            (
                "usdm.private.rest.account.ref",
                r#"{"methods":["balance","positionRisk"]}"#,
            ),
            (
                "usdm.private.rest.listenkey.ref",
                r#"{"lifecycle":"create/extend/delete"}"#,
            ),
        ];

        let adapter = BinanceUsdmRestAdapter::new("https://mock.binance.test");
        for entry in &catalog.rest_endpoints {
            let fixture = fixtures
                .iter()
                .find_map(|(id, payload)| {
                    if *id == entry.id {
                        Some(*payload)
                    } else {
                        None
                    }
                })
                .unwrap();
            let spy = SpyTransport::default();
            spy.enqueue_response(200, fixture);
            let response = adapter
                .execute_rest(
                    &spy,
                    &entry.id,
                    None,
                    if entry.id.contains(".private.") {
                        Some("key-a".to_string())
                    } else {
                        None
                    },
                )
                .await
                .unwrap();

            match response {
                BinanceUsdmRestResponse::GeneralRef(_) => {
                    assert_eq!(entry.id, "usdm.public.rest.general.ref")
                }
                BinanceUsdmRestResponse::ErrorsRef(_) => {
                    assert_eq!(entry.id, "usdm.public.rest.errors.ref")
                }
                BinanceUsdmRestResponse::MarketRef(_) => {
                    assert_eq!(entry.id, "usdm.public.rest.market.ref")
                }
                BinanceUsdmRestResponse::TradeRef(_) => {
                    assert_eq!(entry.id, "usdm.private.rest.trade.ref")
                }
                BinanceUsdmRestResponse::AccountRef(_) => {
                    assert_eq!(entry.id, "usdm.private.rest.account.ref")
                }
                BinanceUsdmRestResponse::ListenKeyRef(_) => {
                    assert_eq!(entry.id, "usdm.private.rest.listenkey.ref")
                }
            }
            assert_eq!(spy.call_count(), 1);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_without_auth_rejects_before_transport() {
        let adapter = BinanceUsdmRestAdapter::new("https://mock.binance.test");
        let spy = SpyTransport::default();
        let err = adapter
            .execute_rest(&spy, "usdm.private.rest.trade.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(spy.call_count(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_flow_never_requires_key() {
        let adapter = BinanceUsdmRestAdapter::new("https://mock.binance.test");
        let spy = SpyTransport::default();
        spy.enqueue_response(200, r#"{"base_rules":{}}"#);
        adapter
            .execute_rest(&spy, "usdm.public.rest.general.ref", None, None)
            .await
            .unwrap();
        assert_eq!(spy.call_count(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn errors_cover_429_5xx_timeout_and_code_mappings() {
        let adapter = BinanceUsdmRestAdapter::new("https://mock.binance.test");

        let spy_429 = SpyTransport::default();
        spy_429.enqueue_response(429, r#"{"retryAfterMs":777}"#);
        let err = adapter
            .execute_rest(&spy_429, "usdm.public.rest.general.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::RateLimited);
        assert_eq!(err.retry_after_ms, Some(777));

        let spy_5xx = SpyTransport::default();
        spy_5xx.enqueue_response(502, "bad gateway");
        let err = adapter
            .execute_rest(&spy_5xx, "usdm.public.rest.general.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Upstream5xx);

        let spy_timeout = SpyTransport::default();
        spy_timeout.enqueue_error(UcelError::new(ErrorCode::Timeout, "timed out"));
        let err = adapter
            .execute_rest(&spy_timeout, "usdm.public.rest.general.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);

        let spy_auth = SpyTransport::default();
        spy_auth.enqueue_response(400, r#"{"code":-2015,"msg":"x"}"#);
        let err = adapter
            .execute_rest(
                &spy_auth,
                "usdm.private.rest.account.ref",
                None,
                Some("key-a".to_string()),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::AuthFailed);

        let spy_permission = SpyTransport::default();
        spy_permission.enqueue_response(403, r#"{"code":-2017,"msg":"x"}"#);
        let err = adapter
            .execute_rest(
                &spy_permission,
                "usdm.private.rest.account.ref",
                None,
                Some("key-a".to_string()),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);

        let spy_invalid = SpyTransport::default();
        spy_invalid.enqueue_response(400, r#"{"code":-1111,"msg":"x"}"#);
        let err = adapter
            .execute_rest(
                &spy_invalid,
                "usdm.private.rest.trade.ref",
                None,
                Some("key-a".to_string()),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidOrder);
    }
}
