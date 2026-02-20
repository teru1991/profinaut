use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ucel_cex_coinbase::{CoinbaseRestAdapter, CoinbaseRestResponse};
use ucel_core::{ErrorCode, UcelError};
use ucel_transport::{
    HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream,
};

#[derive(Clone, Default)]
struct MockTransport {
    responses: Arc<Mutex<VecDeque<Result<HttpResponse, UcelError>>>>,
    calls: Arc<Mutex<Vec<(HttpRequest, RequestContext)>>>,
}

impl MockTransport {
    fn with_responses(responses: Vec<Result<HttpResponse, UcelError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            calls: Arc::new(Mutex::new(vec![])),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl Transport for MockTransport {
    async fn send_http(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.lock().unwrap().push((req, ctx));
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| {
                Ok(HttpResponse {
                    status: 200,
                    body: Bytes::from_static(br#"{"id":"default","source":"fixture"}"#),
                })
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

fn fixture(id: &str) -> Bytes {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{id}.json"));
    Bytes::from(std::fs::read(path).unwrap())
}

#[tokio::test(flavor = "current_thread")]
async fn rest_catalog_all_rows_are_tested_with_typed_parse() {
    let adapter = CoinbaseRestAdapter::new();
    for spec in CoinbaseRestAdapter::endpoint_specs() {
        let transport = if spec.transport_enabled {
            MockTransport::with_responses(vec![Ok(HttpResponse {
                status: 200,
                body: fixture(spec.id),
            })])
        } else {
            MockTransport::default()
        };

        let key_id = if spec.requires_auth {
            Some("k1".to_string())
        } else {
            None
        };
        let resp = adapter
            .execute_rest(&transport, spec.id, None, key_id)
            .await
            .unwrap();

        match resp {
            CoinbaseRestResponse::Reference(r) | CoinbaseRestResponse::ReferenceOnly(r) => {
                assert_eq!(r.id, spec.id);
            }
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn maps_429_5xx_and_timeout() {
    let adapter = CoinbaseRestAdapter::new();

    let t429 = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 429,
        body: Bytes::from_static(br#"{"code":"rate_limited","retry_after_ms":777}"#),
    })]);
    let e429 = adapter
        .execute_rest(
            &t429,
            "advanced.crypto.public.rest.reference.introduction",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(e429.code, ErrorCode::RateLimited);
    assert_eq!(e429.retry_after_ms, Some(777));

    let t500 = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 503,
        body: Bytes::from_static(br#"{"code":"server_error"}"#),
    })]);
    let e500 = adapter
        .execute_rest(
            &t500,
            "advanced.crypto.public.rest.reference.introduction",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(e500.code, ErrorCode::Upstream5xx);

    let tto =
        MockTransport::with_responses(vec![Err(UcelError::new(ErrorCode::Timeout, "timeout"))]);
    let eto = adapter
        .execute_rest(
            &tto,
            "advanced.crypto.public.rest.reference.introduction",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(eto.code, ErrorCode::Timeout);
}

#[tokio::test(flavor = "current_thread")]
async fn private_preflight_reject_does_not_reach_transport() {
    let adapter = CoinbaseRestAdapter::new();
    let transport = MockTransport::default();
    let err = adapter
        .execute_rest(
            &transport,
            "exchange.crypto.private.rest.reference.introduction",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert_eq!(transport.call_count(), 0);
}
