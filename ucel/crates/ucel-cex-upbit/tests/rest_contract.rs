use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ucel_cex_upbit::{UpbitRestAdapter, UpbitRestResponse};
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

    fn last_call(&self) -> Option<(HttpRequest, RequestContext)> {
        self.calls.lock().unwrap().last().cloned()
    }
}

impl Transport for MockTransport {
    async fn send_http(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.lock().unwrap().push((req, ctx));
        self.responses.lock().unwrap().pop_front().unwrap_or_else(|| {
            Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(br#"[{"market":"KRW-BTC","korean_name":"BitcoinKo","english_name":"Bitcoin"}]"#),
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
    let adapter = UpbitRestAdapter::new();

    for spec in adapter.endpoint_specs() {
        let transport = MockTransport::with_responses(vec![Ok(HttpResponse {
            status: 200,
            body: fixture(&spec.id),
        })]);

        let key_id = if spec.requires_auth {
            Some("k1".to_string())
        } else {
            Some("should_be_dropped".to_string())
        };

        let response = adapter
            .execute_rest(&transport, &spec.id, None, key_id)
            .await
            .unwrap();

        match response {
            UpbitRestResponse::Markets(v) => assert!(!v.is_empty()),
            UpbitRestResponse::Tickers(v) => assert!(!v.is_empty()),
            UpbitRestResponse::Trades(v) => assert!(!v.is_empty()),
            UpbitRestResponse::Orderbook(v) => assert!(!v.is_empty()),
            UpbitRestResponse::Candles(v) => assert!(!v.is_empty()),
            UpbitRestResponse::Accounts(v) => assert!(!v.is_empty()),
            UpbitRestResponse::CreateOrder(v) => assert!(!v.uuid.is_empty()),
            UpbitRestResponse::CancelOrder(v) => assert!(!v.uuid.is_empty()),
            UpbitRestResponse::OpenOrders(v) => assert!(!v.is_empty()),
            UpbitRestResponse::ClosedOrders(v) => assert!(!v.is_empty()),
            UpbitRestResponse::OrderChance(v) => assert!(!v.market.id.is_empty()),
            UpbitRestResponse::Withdraws(v) => assert!(!v.is_empty()),
            UpbitRestResponse::WithdrawCoin(v) => assert!(!v.currency.is_empty()),
            UpbitRestResponse::Deposits(v) => assert!(!v.is_empty()),
            UpbitRestResponse::DepositAddress(v) => assert!(!v.deposit_address.is_empty()),
            UpbitRestResponse::TravelRuleVasps(v) => assert!(!v.is_empty()),
            UpbitRestResponse::WalletStatus(v) => assert!(!v.is_empty()),
            UpbitRestResponse::ApiKeys(v) => assert!(!v.is_empty()),
        }

        let (_, ctx) = transport.last_call().unwrap();
        if spec.requires_auth {
            assert_eq!(ctx.key_id.as_deref(), Some("k1"));
        } else {
            assert!(ctx.key_id.is_none(), "public endpoint must not carry key path");
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn maps_429_5xx_and_timeout() {
    let adapter = UpbitRestAdapter::new();

    let t429 = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 429,
        body: Bytes::from_static(br#"{"error":{"name":"too_many_requests"},"retry_after_ms":777}"#),
    })]);
    let e429 = adapter
        .execute_rest(&t429, "quotation.public.rest.markets.list", None, None)
        .await
        .unwrap_err();
    assert_eq!(e429.code, ErrorCode::RateLimited);
    assert_eq!(e429.retry_after_ms, Some(777));

    let t500 = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 503,
        body: Bytes::from_static(br#"{"error":{"name":"server_error"}}"#),
    })]);
    let e500 = adapter
        .execute_rest(&t500, "quotation.public.rest.markets.list", None, None)
        .await
        .unwrap_err();
    assert_eq!(e500.code, ErrorCode::Upstream5xx);

    let tto = MockTransport::with_responses(vec![Err(UcelError::new(ErrorCode::Timeout, "timeout"))]);
    let eto = adapter
        .execute_rest(&tto, "quotation.public.rest.markets.list", None, None)
        .await
        .unwrap_err();
    assert_eq!(eto.code, ErrorCode::Timeout);
}

#[tokio::test(flavor = "current_thread")]
async fn private_preflight_reject_does_not_reach_transport() {
    let adapter = UpbitRestAdapter::new();
    let transport = MockTransport::default();

    let err = adapter
        .execute_rest(&transport, "exchange.private.rest.accounts.list", None, None)
        .await
        .unwrap_err();

    assert_eq!(err.code, ErrorCode::MissingAuth);
    assert_eq!(transport.call_count(), 0);
}
