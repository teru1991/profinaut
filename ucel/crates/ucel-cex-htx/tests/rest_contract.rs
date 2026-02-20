use bytes::Bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use ucel_cex_htx::{
    log_private_ws_auth_attempt, map_htx_http_error, EndpointSpec, HtxBackpressure, HtxRestAdapter,
    HtxWsAdapter, OrderBookHealth, OrderBookResyncEngine, WsCounters, WsSubscription,
};
use ucel_core::{ErrorCode, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, UcelError};
use ucel_transport::{
    HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream,
};

#[derive(Default)]
struct SpyTransport {
    calls: AtomicUsize,
    ws_calls: AtomicUsize,
    queued_responses: Mutex<Vec<(u16, String)>>,
    passthrough_err: Mutex<Option<UcelError>>,
    seen_key_ids: AsyncMutex<Vec<Option<String>>>,
    ws_seen_key_ids: AsyncMutex<Vec<Option<String>>>,
    ws_urls: AsyncMutex<Vec<String>>,
    responses: AsyncMutex<HashMap<String, HttpResponse>>,
}

impl SpyTransport {
    fn with_response(status: u16, body: impl Into<String>) -> Self {
        let queued_responses = vec![(status, body.into())];
        Self {
            calls: AtomicUsize::new(0),
            ws_calls: AtomicUsize::new(0),
            queued_responses: Mutex::new(queued_responses),
            passthrough_err: Mutex::new(None),
            seen_key_ids: AsyncMutex::new(vec![]),
            ws_seen_key_ids: AsyncMutex::new(vec![]),
            ws_urls: AsyncMutex::new(vec![]),
            responses: AsyncMutex::new(HashMap::new()),
        }
    }

    fn with_error(err: UcelError) -> Self {
        Self {
            passthrough_err: Mutex::new(Some(err)),
            ..Default::default()
        }
    }
}

impl Transport for SpyTransport {
    async fn send_http(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.seen_key_ids.lock().await.push(ctx.key_id);

        if let Some(err) = self.passthrough_err.lock().unwrap().take() {
            return Err(err);
        }

        if let Some(resp) = self.responses.lock().await.remove(&req.path) {
            return Ok(resp);
        }

        let (status, body) = self
            .queued_responses
            .lock()
            .unwrap()
            .pop()
            .expect("queued response");
        Ok(HttpResponse {
            status,
            body: Bytes::from(body),
        })
    }

    async fn connect_ws(
        &self,
        req: WsConnectRequest,
        ctx: RequestContext,
    ) -> Result<WsStream, UcelError> {
        self.ws_calls.fetch_add(1, Ordering::SeqCst);
        self.ws_seen_key_ids.lock().await.push(ctx.key_id);
        self.ws_urls.lock().await.push(req.url);
        Ok(WsStream::default())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn contract_test_all_rest_ids_parse_fixture() {
    let adapter = HtxRestAdapter::new();
    let fixture = include_str!("fixtures/rest.success.json").to_string();

    for spec in adapter.endpoints.iter() {
        let transport = SpyTransport::with_response(200, fixture.clone());
        let key = spec.requires_auth.then_some("k".to_string());
        let out = adapter
            .execute_rest(&transport, &spec.id, None, key)
            .await
            .unwrap();
        assert_eq!(out.status.as_deref(), Some("ok"), "id={}", spec.id);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn private_preflight_rejects_without_transport_hit() {
    let adapter = HtxRestAdapter::from_specs(vec![EndpointSpec {
        id: "spot.private.rest.x".into(),
        method: "GET".into(),
        base_url: "https://api.htx.com".into(),
        path: "/x".into(),
        requires_auth: true,
    }]);

    let transport = SpyTransport::default();
    let err = adapter
        .execute_rest(&transport, "spot.private.rest.x", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn public_ops_do_not_route_keys() {
    let adapter = HtxRestAdapter::from_specs(vec![EndpointSpec {
        id: "spot.public.rest.x".into(),
        method: "GET".into(),
        base_url: "https://api.htx.com".into(),
        path: "/x".into(),
        requires_auth: false,
    }]);
    let transport = SpyTransport::with_response(200, "{\"status\":\"ok\"}");

    adapter
        .execute_rest(
            &transport,
            "spot.public.rest.x",
            None,
            Some("secret-key".into()),
        )
        .await
        .unwrap();

    assert_eq!(transport.seen_key_ids.lock().await.as_slice(), &[None]);
}

#[tokio::test(flavor = "current_thread")]
async fn maps_429_5xx_timeout() {
    let rate = map_htx_http_error(429, b"retry_after_ms=1500");
    assert_eq!(rate.code, ErrorCode::RateLimited);
    assert_eq!(rate.retry_after_ms, Some(1500));

    let upstream = map_htx_http_error(502, br#"{}"#);
    assert_eq!(upstream.code, ErrorCode::Upstream5xx);

    let adapter = HtxRestAdapter::from_specs(vec![EndpointSpec {
        id: "spot.public.rest.x".into(),
        method: "GET".into(),
        base_url: "https://api.htx.com".into(),
        path: "/x".into(),
        requires_auth: false,
    }]);
    let transport = SpyTransport::with_error(UcelError::new(ErrorCode::Timeout, "timeout"));
    let err = adapter
        .execute_rest(&transport, "spot.public.rest.x", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Timeout);
}

#[test]
fn maps_auth_permission_invalid_order_by_code_field() {
    let auth = map_htx_http_error(401, br#"{\"err-code\":\"api-signature-not-valid\"}"#);
    assert_eq!(auth.code, ErrorCode::AuthFailed);

    let perm = map_htx_http_error(403, br#"{\"error-code\":\"permission-denied\"}"#);
    assert_eq!(perm.code, ErrorCode::PermissionDenied);

    let invalid = map_htx_http_error(422, br#"{\"code\":\"order-invalid\"}"#);
    assert_eq!(invalid.code, ErrorCode::InvalidOrder);
}

fn ws_fixture_for(id: &str) -> &'static str {
    match id {
        "spot.private.ws.account.catalog.index"
        | "futures.private.ws.account.catalog.index"
        | "swap.private.ws.account.catalog.index" => {
            "{\"op\":\"notify\",\"topic\":\"orders.btcusdt\"}"
        }
        "other.public.ws.common.protocol" => {
            "{\"action\":\"push\",\"ch\":\"market.btcusdt.kline.1min\",\"status\":\"ok\"}"
        }
        "other.public.ws.other.not_applicable" => "{\"status\":\"ok\"}",
        _ => "{\"ch\":\"market.btcusdt.depth.step0\",\"ts\":1700000000000,\"tick\":{\"bids\":[[100.0,1.0]],\"asks\":[[101.0,1.5]]}}",
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ws_contract_all_catalog_ids_parse_and_build_commands() {
    let ws = HtxWsAdapter::new(Arc::new(WsCounters::default()));
    for spec in ws.channel_specs() {
        let sub = WsSubscription {
            channel_id: spec.id.clone(),
            symbol: Some("btcusdt".into()),
            contract_code: Some("BTC-USDT".into()),
            topic: Some("depth.step0".into()),
            channel: Some("market.btcusdt.kline.1min".into()),
            key_id: spec.requires_auth.then_some("kid-1".to_string()),
        };
        let sub_cmd = ws.build_subscribe_command(&sub).unwrap();
        let unsub_cmd = ws.build_unsubscribe_command(&sub).unwrap();
        assert!(!sub_cmd.payload.contains('$'));
        assert!(!unsub_cmd.payload.contains('$'));

        let body = Bytes::from_static(ws_fixture_for(&spec.id).as_bytes());
        assert!(
            ws.parse_market_event(&spec.id, &body).is_ok(),
            "id={}",
            spec.id
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ws_reconnect_resubscribe_and_idempotent() {
    let transport = SpyTransport::default();
    let counters = Arc::new(WsCounters::default());
    let ws = HtxWsAdapter::new(counters.clone());

    let sub = WsSubscription {
        channel_id: "spot.public.ws.market.catalog.index".into(),
        symbol: Some("btcusdt".into()),
        contract_code: None,
        topic: Some("depth.step0".into()),
        channel: None,
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
async fn ws_private_preflight_reject_without_connect_and_public_keyless() {
    let transport = SpyTransport::default();
    let ws = HtxWsAdapter::new(Arc::new(WsCounters::default()));

    let private = WsSubscription {
        channel_id: "spot.private.ws.account.catalog.index".into(),
        symbol: Some("btcusdt".into()),
        contract_code: None,
        topic: Some("depth.step0".into()),
        channel: None,
        key_id: None,
    };
    let err = ws.subscribe(&transport, private).await.unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert_eq!(transport.ws_calls.load(Ordering::SeqCst), 0);

    let public = WsSubscription {
        channel_id: "spot.public.ws.market.catalog.index".into(),
        symbol: Some("btcusdt".into()),
        contract_code: None,
        topic: Some("trade.detail".into()),
        channel: None,
        key_id: Some("should-not-pass".into()),
    };
    ws.subscribe(&transport, public).await.unwrap();
    assert_eq!(transport.ws_seen_key_ids.lock().await.as_slice(), &[None]);
}

#[tokio::test(flavor = "current_thread")]
async fn backpressure_bounded_channel_drops_and_metrics() {
    let counters = Arc::new(WsCounters::default());
    let mut bp = HtxBackpressure::new(1, counters.clone());
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
    use tracing_subscriber::fmt::MakeWriter;

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

#[test]
fn strict_coverage_gate_has_no_gaps_and_includes_all_rest_ws_ids() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let raw_catalog =
        std::fs::read_to_string(repo_root.join("docs/exchanges/htx/catalog.json")).unwrap();
    let catalog: serde_json::Value = serde_json::from_str(&raw_catalog).unwrap();

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
    catalog_ids.sort_unstable();

    let manifest_path = repo_root.join("ucel/coverage/htx.yaml");
    let raw_manifest = std::fs::read_to_string(manifest_path).unwrap();
    let manifest: CoverageManifest = serde_yaml::from_str(&raw_manifest).unwrap();
    assert_eq!(manifest.venue, "htx");
    assert!(manifest.strict);

    let mut covered = vec![];
    for entry in &manifest.entries {
        assert!(entry.implemented, "id not implemented: {}", entry.id);
        assert!(entry.tested, "id not tested: {}", entry.id);
        covered.push(entry.id.clone());
    }
    covered.sort_unstable();
    assert_eq!(covered, catalog_ids);
}
