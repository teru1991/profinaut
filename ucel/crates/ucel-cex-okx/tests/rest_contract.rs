use bytes::Bytes;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use ucel_cex_okx::{map_okx_http_error, OkxRestAdapter, OkxRestResponse};
use ucel_core::{ErrorCode, UcelError};
use ucel_transport::{
    HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream,
};

#[derive(Default)]
struct MockTransport {
    calls: AtomicUsize,
    response: Mutex<Option<Result<HttpResponse, UcelError>>>,
}

impl MockTransport {
    fn with_response(response: Result<HttpResponse, UcelError>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            response: Mutex::new(Some(response)),
        }
    }
}

impl Transport for MockTransport {
    async fn send_http(
        &self,
        _req: HttpRequest,
        _ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
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

fn fixture_for(id: &str) -> &'static [u8] {
    match id {
        "okx.rest.overview" => include_bytes!("fixtures/okx.rest.overview.json"),
        "okx.rest.auth" => include_bytes!("fixtures/okx.rest.auth.json"),
        "okx.rest.public" => include_bytes!("fixtures/okx.rest.public.json"),
        "okx.rest.private" => include_bytes!("fixtures/okx.rest.private.json"),
        _ => panic!("missing fixture for {id}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rest_catalog_all_rows_parse_from_fixtures() {
    let adapter = OkxRestAdapter::new();

    for spec in adapter.endpoint_specs() {
        let transport = MockTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::copy_from_slice(fixture_for(&spec.id)),
        }));
        let key = if spec.requires_auth {
            Some("k".into())
        } else {
            None
        };
        let out = adapter
            .execute_rest(&transport, &spec.id, None, key)
            .await
            .unwrap();
        assert!(
            matches!(out, OkxRestResponse::Envelope(_)),
            "id={} should parse",
            spec.id
        );
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rest_contract_429_5xx_timeout_and_private_preflight() {
    let adapter = OkxRestAdapter::new();

    let t429 = MockTransport::with_response(Ok(HttpResponse {
        status: 429,
        body: Bytes::from_static(b"retry_after_ms=1600"),
    }));
    let err = adapter
        .execute_rest(&t429, "okx.rest.public", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::RateLimited);
    assert_eq!(err.retry_after_ms, Some(1600));

    let t5xx = MockTransport::with_response(Ok(HttpResponse {
        status: 503,
        body: Bytes::from_static(br#"{}"#),
    }));
    let err = adapter
        .execute_rest(&t5xx, "okx.rest.public", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Upstream5xx);

    let timeout = MockTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "timeout")));
    let err = adapter
        .execute_rest(&timeout, "okx.rest.public", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Timeout);

    let preflight = MockTransport::default();
    let err = adapter
        .execute_rest(&preflight, "okx.rest.private", None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert_eq!(preflight.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn maps_okx_error_codes_without_message_branching() {
    let e = map_okx_http_error(401, br#"{"code":"50113","msg":"Invalid signature"}"#);
    assert_eq!(e.code, ErrorCode::AuthFailed);

    let e = map_okx_http_error(403, br#"{"code":"50035","msg":"No permission"}"#);
    assert_eq!(e.code, ErrorCode::PermissionDenied);

    let e = map_okx_http_error(400, br#"{"code":"51008","msg":"Insufficient balance"}"#);
    assert_eq!(e.code, ErrorCode::InvalidOrder);
}
