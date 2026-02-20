use bytes::Bytes;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use ucel_cex_deribit::*;
use ucel_core::{ErrorCode, UcelError};
use ucel_transport::{HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream};

#[derive(Default)]
struct SpyTransport {
    calls: AtomicUsize,
    response: Mutex<Option<Result<HttpResponse, UcelError>>>,
}

impl SpyTransport {
    fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            response: Mutex::new(Some(resp)),
        }
    }
}

impl Transport for SpyTransport {
    async fn send_http(&self, _req: HttpRequest, _ctx: RequestContext) -> Result<HttpResponse, UcelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.response.lock().unwrap().take().unwrap()
    }

    async fn connect_ws(&self, _req: WsConnectRequest, _ctx: RequestContext) -> Result<WsStream, UcelError> {
        Ok(WsStream::default())
    }
}

fn adapter() -> DeribitRestAdapter {
    DeribitRestAdapter::new("https://www.deribit.com/api/v2")
}

#[tokio::test(flavor = "current_thread")]
async fn rest_catalog_all_ids_have_success_parse_fixture() {
    let fixtures = vec![
        (DeribitRestRequest::PublicGetInstruments(PublicGetInstrumentsParams { currency: "BTC".into(), kind: None, expired: None }), br#"{"jsonrpc":"2.0","id":1,"result":[{"instrument_name":"BTC-PERPETUAL","kind":"future","base_currency":"BTC","quote_currency":"USD","settlement_currency":"BTC","tick_size":0.5}]}"#.to_vec(), None),
        (DeribitRestRequest::PublicTicker(PublicTickerParams { instrument_name: "BTC-PERPETUAL".into() }), br#"{"jsonrpc":"2.0","id":1,"result":{"instrument_name":"BTC-PERPETUAL","timestamp":1}}"#.to_vec(), None),
        (DeribitRestRequest::PublicGetOrderBook(PublicGetOrderBookParams { instrument_name: "BTC-PERPETUAL".into(), depth: Some(5) }), br#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":1,"instrument_name":"BTC-PERPETUAL","bids":[[1.0,2.0]],"asks":[[2.0,1.0]]}}"#.to_vec(), None),
        (DeribitRestRequest::PublicGetTradingViewChartData(PublicGetTradingViewChartDataParams { instrument_name: "BTC-PERPETUAL".into(), start_timestamp: 1, end_timestamp: 2, resolution: "1".into() }), br#"{"jsonrpc":"2.0","id":1,"result":{"ticks":[1],"open":[1.0],"high":[1.0],"low":[1.0],"close":[1.0],"volume":[1.0]}}"#.to_vec(), None),
        (DeribitRestRequest::PublicAuth(PublicAuthParams { grant_type: "client_credentials".into(), client_id: Some("id".into()), client_secret: Some("sec".into()) }), br#"{"jsonrpc":"2.0","id":1,"result":{"access_token":"a","expires_in":10,"token_type":"bearer"}}"#.to_vec(), None),
        (DeribitRestRequest::PrivateGetAccountSummary(PrivateGetAccountSummaryParams { currency: "BTC".into(), extended: None }), br#"{"jsonrpc":"2.0","id":1,"result":{"currency":"BTC","balance":1.0}}"#.to_vec(), Some("k".to_string())),
        (DeribitRestRequest::PrivateBuy(PrivateOrderParams { instrument_name: "BTC-PERPETUAL".into(), amount: 1.0, order_type: "limit".into(), price: Some(1.0) }), br#"{"jsonrpc":"2.0","id":1,"result":{"order_id":"o1","instrument_name":"BTC-PERPETUAL"}}"#.to_vec(), Some("k".to_string())),
        (DeribitRestRequest::PrivateSell(PrivateOrderParams { instrument_name: "BTC-PERPETUAL".into(), amount: 1.0, order_type: "limit".into(), price: Some(1.0) }), br#"{"jsonrpc":"2.0","id":1,"result":{"order_id":"o2","instrument_name":"BTC-PERPETUAL"}}"#.to_vec(), Some("k".to_string())),
        (DeribitRestRequest::PrivateCancel(PrivateCancelParams { order_id: "o1".into() }), br#"{"jsonrpc":"2.0","id":1,"result":{"order_id":"o1"}}"#.to_vec(), Some("k".to_string())),
    ];

    for (req, body, key) in fixtures {
        let transport = SpyTransport::with_response(Ok(HttpResponse { status: 200, body: Bytes::from(body) }));
        let out = adapter().execute_rest(&transport, req, key).await;
        assert!(out.is_ok());
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rest_handles_429_5xx_timeout_and_preflight_reject() {
    let a = adapter();

    let t429 = SpyTransport::with_response(Ok(HttpResponse { status: 429, body: Bytes::from_static(b"retry_after_ms=123") }));
    let e429 = a.execute_rest(&t429, DeribitRestRequest::PublicTicker(PublicTickerParams { instrument_name: "BTC-PERPETUAL".into() }), None).await.unwrap_err();
    assert_eq!(e429.code, ErrorCode::RateLimited);
    assert_eq!(e429.retry_after_ms, Some(123));

    let t5 = SpyTransport::with_response(Ok(HttpResponse { status: 503, body: Bytes::new() }));
    let e5 = a.execute_rest(&t5, DeribitRestRequest::PublicTicker(PublicTickerParams { instrument_name: "BTC-PERPETUAL".into() }), None).await.unwrap_err();
    assert_eq!(e5.code, ErrorCode::Upstream5xx);

    let tt = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "timeout")));
    let et = a.execute_rest(&tt, DeribitRestRequest::PublicTicker(PublicTickerParams { instrument_name: "BTC-PERPETUAL".into() }), None).await.unwrap_err();
    assert_eq!(et.code, ErrorCode::Timeout);

    let tp = SpyTransport::with_response(Ok(HttpResponse { status: 200, body: Bytes::new() }));
    let ep = a.execute_rest(&tp, DeribitRestRequest::PrivateCancel(PrivateCancelParams { order_id: "x".into() }), None).await.unwrap_err();
    assert_eq!(ep.code, ErrorCode::MissingAuth);
    assert_eq!(tp.calls.load(Ordering::SeqCst), 0);
}
