use bytes::Bytes;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use ucel_cex_htx::{map_htx_http_error, EndpointSpec, HtxRestAdapter};
use ucel_core::{ErrorCode, UcelError};
use ucel_testkit::RestMockServer;
use ucel_transport::{HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream};

#[derive(Default)]
struct SpyTransport {
    calls: AtomicUsize,
    server: Mutex<RestMockServer>,
    passthrough_err: Mutex<Option<UcelError>>,
    seen_key_ids: Mutex<Vec<Option<String>>>,
}

impl SpyTransport {
    fn with_response(status: u16, body: impl Into<String>) -> Self {
        let mut server = RestMockServer::default();
        server.enqueue(status, body);
        Self {
            calls: AtomicUsize::new(0),
            server: Mutex::new(server),
            passthrough_err: Mutex::new(None),
            seen_key_ids: Mutex::new(vec![]),
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
        _req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.seen_key_ids.lock().unwrap().push(ctx.key_id);

        if let Some(err) = self.passthrough_err.lock().unwrap().take() {
            return Err(err);
        }

        let (status, body) = self
            .server
            .lock()
            .unwrap()
            .next_response()
            .expect("queued response");
        Ok(HttpResponse {
            status,
            body: Bytes::from(body),
        })
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
        .execute_rest(&transport, "spot.public.rest.x", None, Some("secret-key".into()))
        .await
        .unwrap();

    assert_eq!(transport.seen_key_ids.lock().unwrap().as_slice(), &[None]);
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
