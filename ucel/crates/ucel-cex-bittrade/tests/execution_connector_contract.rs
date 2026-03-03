use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ucel_cex_bittrade::execution::BittradeExecutionConnector;
use ucel_core::{ErrorCode, UcelError};
use ucel_sdk::execution::*;
use ucel_transport::{
    HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream,
};

/// MockTransport: 順序付きレスポンスキューと呼び出し記録を持つ
#[derive(Default)]
struct MockTransport {
    calls: Mutex<Vec<HttpRequest>>,
    responses: Mutex<VecDeque<Result<HttpResponse, UcelError>>>,
}

impl MockTransport {
    fn with_responses(resps: Vec<Result<HttpResponse, UcelError>>) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(vec![]),
            responses: Mutex::new(resps.into_iter().collect()),
        })
    }

    fn take_calls(&self) -> Vec<HttpRequest> {
        self.calls.lock().unwrap().clone()
    }
}

impl Transport for MockTransport {
    async fn send_http(
        &self,
        req: HttpRequest,
        _ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.lock().unwrap().push(req);
        let mut g = self.responses.lock().unwrap();
        g.pop_front()
            .unwrap_or_else(|| Err(UcelError::new(ErrorCode::Internal, "no mock response")))
    }

    async fn connect_ws(
        &self,
        _req: WsConnectRequest,
        _ctx: RequestContext,
    ) -> Result<WsStream, UcelError> {
        Ok(WsStream { connected: false })
    }
}

fn mk_intent() -> OrderIntent {
    OrderIntent {
        intent_id: OrderIntentId::new("intent-001"),
        venue: VenueId::new("bittrade"),
        symbol: Symbol::new("BTCJPY"),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        tif: Some(OrderTimeInForce::Gtc),
        price: Some(Price(100.0)),
        qty: Quantity(0.01),
        tags: std::collections::BTreeMap::new(),
    }
}

/// accounts.get → order.place の順で呼ばれ、body に client-order-id が入ることを確認
#[tokio::test]
async fn bittrade_connector_places_with_account_id_and_client_order_id() {
    let transport = MockTransport::with_responses(vec![
        // 1) accounts.get
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":[{"id":12345}]}"#),
        }),
        // 2) order.place
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":"999"}"#),
        }),
    ]);
    let connector = BittradeExecutionConnector::new(transport.clone(), "key-1");

    let mut req = OrderRequest {
        mode: ExecutionMode::Live,
        intent: mk_intent(),
        idempotency: IdempotencyKey::parse("idem-1234567890abcdef").unwrap(),
        run_id: Some("run-1".to_string()),
    };
    // SDK が Live で注入するが、connector 単体テストなので明示注入
    req.intent
        .tags
        .insert("client_order_id".to_string(), req.idempotency.0.clone());

    let receipt = connector.place_order(&req).await.unwrap();
    assert_eq!(receipt.venue_order_id.as_deref(), Some("999"));
    assert_eq!(receipt.status, OrderStatus::Accepted);

    let calls = transport.take_calls();
    assert_eq!(calls.len(), 2);

    // 1st call: accounts.get
    assert!(
        calls[0].path.contains("/v1/account/accounts"),
        "expected accounts path, got: {}",
        calls[0].path
    );

    // 2nd call: order.place
    assert!(
        calls[1].path.contains("/v1/order/orders/place"),
        "expected place path, got: {}",
        calls[1].path
    );
    let body_str = String::from_utf8(calls[1].body.clone().unwrap_or_default().to_vec()).unwrap();
    assert!(
        body_str.contains("12345"),
        "expected account-id 12345 in body: {body_str}"
    );
    assert!(
        body_str.contains("idem-1234567890abcdef"),
        "expected client-order-id in body: {body_str}"
    );
}

/// account-id がキャッシュされ、2 回目の place では accounts.get を呼ばない
#[tokio::test]
async fn bittrade_connector_caches_account_id() {
    let transport = MockTransport::with_responses(vec![
        // accounts.get（1回だけ）
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":[{"id":99}]}"#),
        }),
        // place 1 回目
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":"111"}"#),
        }),
        // place 2 回目（accounts.get は呼ばれないはず）
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":"222"}"#),
        }),
    ]);
    let connector = BittradeExecutionConnector::new(transport.clone(), "key-1");

    for _ in 0..2 {
        let mut req = OrderRequest {
            mode: ExecutionMode::Live,
            intent: mk_intent(),
            idempotency: IdempotencyKey::random_uuid(),
            run_id: None,
        };
        req.intent
            .tags
            .insert("client_order_id".to_string(), req.idempotency.0.clone());
        connector.place_order(&req).await.unwrap();
    }

    let calls = transport.take_calls();
    // accounts(1) + place(2) = 3 calls
    assert_eq!(
        calls.len(),
        3,
        "expected 3 calls (1 accounts + 2 places), got {}",
        calls.len()
    );
    assert!(calls[0].path.contains("/v1/account/accounts"));
    assert!(calls[1].path.contains("/v1/order/orders/place"));
    assert!(calls[2].path.contains("/v1/order/orders/place"));
}

/// cancel が order-id を path に正しく埋める
#[tokio::test]
async fn bittrade_connector_cancel_sends_correct_path() {
    let transport = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 200,
        body: Bytes::from(r#"{"status":"ok","data":"888"}"#),
    })]);
    let connector = BittradeExecutionConnector::new(transport.clone(), "key-1");

    let cancel = OrderCancel {
        venue: VenueId::new("bittrade"),
        symbol: Symbol::new("BTCJPY"),
        venue_order_id: "ORDER-777".to_string(),
        idempotency: IdempotencyKey::parse("cancel-idem-1234567890abc").unwrap(),
        run_id: None,
    };
    let ok = connector.cancel_order(&cancel).await.unwrap();
    assert!(ok);

    let calls = transport.take_calls();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].path.contains("ORDER-777"),
        "expected order-id ORDER-777 in path: {}",
        calls[0].path
    );
    assert!(
        calls[0].path.contains("submitcancel"),
        "expected submitcancel in path: {}",
        calls[0].path
    );
}

/// list_open_orders は states クエリパラメータを含む
#[tokio::test]
async fn bittrade_connector_list_open_orders_sends_states_param() {
    let transport = MockTransport::with_responses(vec![Ok(HttpResponse {
        status: 200,
        body: Bytes::from(
            r#"{"status":"ok","data":[{"id":555,"symbol":"btcjpy","type":"buy-limit"}]}"#,
        ),
    })]);
    let connector = BittradeExecutionConnector::new(transport.clone(), "key-1");

    let q = OrderOpenQuery {
        venue: VenueId::new("bittrade"),
        symbol: Some(Symbol::new("BTCJPY")),
    };
    let orders = connector.list_open_orders(&q).await.unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].venue_order_id.as_deref(), Some("555"));

    let calls = transport.take_calls();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].path.contains("states=submitted"),
        "expected states query param: {}",
        calls[0].path
    );
    assert!(
        calls[0].path.contains("btcjpy"),
        "expected symbol btcjpy: {}",
        calls[0].path
    );
}

/// reconcile は ok=true を返す（v1 最小実装）
#[tokio::test]
async fn bittrade_connector_reconcile_returns_ok() {
    let transport = MockTransport::with_responses(vec![]);
    let connector = BittradeExecutionConnector::new(transport, "key-1");

    let report = connector
        .reconcile(&VenueId::new("bittrade"))
        .await
        .unwrap();
    assert!(report.ok);
    assert!(report.mismatches.is_empty());
}

/// ExecutionClientAsync（唯一の出口）経由の place で client_order_id が自動注入される
#[tokio::test]
async fn execution_client_async_injects_client_order_id_for_live() {
    let transport = MockTransport::with_responses(vec![
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":[{"id":1}]}"#),
        }),
        Ok(HttpResponse {
            status: 200,
            body: Bytes::from(r#"{"status":"ok","data":"42"}"#),
        }),
    ]);
    let connector = BittradeExecutionConnector::new(transport.clone(), "key-x");
    let client = ExecutionClientAsync::new(connector);

    let req = OrderRequest {
        mode: ExecutionMode::Live,
        intent: mk_intent(), // tags は空
        idempotency: IdempotencyKey::parse("idem-inject-test-123456789").unwrap(),
        run_id: None,
    };
    let out = client.place(req).await.unwrap();
    assert_eq!(out.receipt.venue_order_id.as_deref(), Some("42"));
    // client_order_id は SDK が注入したもの
    assert_eq!(
        out.receipt.client_order_id.as_deref(),
        Some("idem-inject-test-123456789")
    );

    // body に client-order-id が含まれていることを確認
    let calls = transport.take_calls();
    let body_str = String::from_utf8(
        calls
            .get(1)
            .and_then(|c| c.body.clone())
            .unwrap_or_default()
            .to_vec(),
    )
    .unwrap();
    assert!(
        body_str.contains("idem-inject-test-123456789"),
        "expected injected client-order-id in body: {body_str}"
    );
}
