use bytes::Bytes;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use ucel_cex_bittrade::{BittradeRestClient, RequestArgs, REST_ENDPOINTS};
use ucel_core::{ErrorCode, UcelError};
use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
use ucel_transport::{HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream};

#[derive(Clone, Default)]
struct SpyTransport {
    http_calls: Arc<Mutex<Vec<RequestContext>>>,
    queue: Arc<Mutex<VecDeque<Result<HttpResponse, UcelError>>>>,
}

impl SpyTransport {
    fn enqueue_ok(&self, status: u16, body: Bytes) {
        self.queue.lock().unwrap().push_back(Ok(HttpResponse { status, body }));
    }
    fn enqueue_err(&self, err: UcelError) {
        self.queue.lock().unwrap().push_back(Err(err));
    }
}

impl Transport for SpyTransport {
    async fn send_http(&self, _req: HttpRequest, ctx: RequestContext) -> Result<HttpResponse, UcelError> {
        self.http_calls.lock().unwrap().push(ctx);
        self.queue.lock().unwrap().pop_front().unwrap_or_else(|| Ok(HttpResponse { status: 500, body: Bytes::from_static(br#"{"code":"500"}"#) }))
    }

    async fn connect_ws(&self, _req: WsConnectRequest, _ctx: RequestContext) -> Result<WsStream, UcelError> {
        Ok(WsStream { connected: true })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rest_contract_all_catalog_rows_parse() {
    let transport = Arc::new(SpyTransport::default());
    let client = BittradeRestClient::new(transport.clone());

    for spec in REST_ENDPOINTS {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(format!("{}.json", spec.id));
        let payload = std::fs::read(&fixture_path).unwrap();
        transport.enqueue_ok(200, Bytes::from(payload));

        let mut args = RequestArgs::default();
        if spec.path.contains("{account-id}") {
            args.path_params.insert("account-id".into(), "1".into());
        }
        if spec.path.contains("{order-id}") {
            args.path_params.insert("order-id".into(), "2".into());
        }
        if spec.path.contains("{withdraw-id}") {
            args.path_params.insert("withdraw-id".into(), "3".into());
        }

        let key = if spec.requires_auth { Some("k1".into()) } else { None };
        let out = client.execute(spec.id, args, key).await;
        assert!(out.is_ok(), "{}", spec.id);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn private_preflight_rejects_without_transport_call() {
    let transport = Arc::new(SpyTransport::default());
    let client = BittradeRestClient::new(transport.clone());
    let err = client
        .execute("private.rest.order.get", RequestArgs { path_params: BTreeMap::from([("order-id".into(), "1".into())]), ..Default::default() }, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert!(transport.http_calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn public_context_never_uses_key_path() {
    let transport = Arc::new(SpyTransport::default());
    let client = BittradeRestClient::new(transport.clone());
    transport.enqueue_ok(200, Bytes::from_static(br#"{"status":"ok","data":["BTC"]}"#));

    client
        .execute("public.rest.common.currencys.get", RequestArgs::default(), Some("should_drop".into()))
        .await
        .unwrap();

    let calls = transport.http_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].key_id, None);
}

#[tokio::test(flavor = "current_thread")]
async fn maps_429_with_retry_after_and_stops() {
    let transport = Arc::new(SpyTransport::default());
    let mut client = BittradeRestClient::new(transport.clone());
    client.max_retries = 0;
    transport.enqueue_ok(429, Bytes::from_static(br#"{"code":"429","message":"limited","retry_after_ms":123}"#));

    let err = client.execute("public.rest.common.symbols.get", RequestArgs::default(), None).await.unwrap_err();
    assert_eq!(err.code, ErrorCode::RateLimited);
    assert_eq!(err.retry_after_ms, Some(123));
}

#[tokio::test(flavor = "current_thread")]
async fn maps_5xx() {
    let transport = Arc::new(SpyTransport::default());
    let mut client = BittradeRestClient::new(transport.clone());
    client.max_retries = 0;
    transport.enqueue_ok(503, Bytes::from_static(br#"{}"#));

    let err = client.execute("public.rest.common.symbols.get", RequestArgs::default(), None).await.unwrap_err();
    assert_eq!(err.code, ErrorCode::Upstream5xx);
}

#[tokio::test(flavor = "current_thread")]
async fn maps_timeout_transport_error() {
    let transport = Arc::new(SpyTransport::default());
    let mut client = BittradeRestClient::new(transport.clone());
    client.max_retries = 0;
    transport.enqueue_err(UcelError::new(ErrorCode::Timeout, "timeout"));

    let err = client.execute("public.rest.common.symbols.get", RequestArgs::default(), None).await.unwrap_err();
    assert_eq!(err.code, ErrorCode::Timeout);
}

#[test]
fn strict_coverage_gate_for_bittrade_is_zero_gap() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bittrade.yaml");
    let manifest = load_coverage_manifest(&manifest_path).unwrap();
    assert!(manifest.strict);
    let gaps = evaluate_coverage_gate(&manifest);
    assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
}
