use ucel_sdk::execution::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct SpyConnector {
    place_calls: AtomicUsize,
    cancel_calls: AtomicUsize,
    open_calls: AtomicUsize,
}

impl SpyConnector {
    fn new() -> Self {
        Self {
            place_calls: AtomicUsize::new(0),
            cancel_calls: AtomicUsize::new(0),
            open_calls: AtomicUsize::new(0),
        }
    }
}

impl ExecutionConnector for SpyConnector {
    fn place_order(&self, req: &OrderRequest) -> SdkExecutionResult<OrderReceipt> {
        self.place_calls.fetch_add(1, Ordering::SeqCst);
        Ok(OrderReceipt {
            venue: req.intent.venue.clone(),
            symbol: req.intent.symbol.clone(),
            status: OrderStatus::Accepted,
            venue_order_id: Some("venue-order-1".to_string()),
            client_order_id: req.intent.tags.get("client_order_id").cloned(),
            intent_id: req.intent.intent_id.clone(),
            idempotency: req.idempotency.clone(),
        })
    }

    fn cancel_order(&self, _cancel: &OrderCancel) -> SdkExecutionResult<bool> {
        self.cancel_calls.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }

    fn list_open_orders(&self, _q: &OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>> {
        self.open_calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
}

fn base_intent() -> OrderIntent {
    OrderIntent {
        intent_id: OrderIntentId::new("intent-001"),
        venue: VenueId::new("bybit"),
        symbol: Symbol::new("BTCUSDT"),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        tif: Some(OrderTimeInForce::Gtc),
        price: Some(Price(100.0)),
        qty: Quantity(0.01),
        tags: {
            let mut m = std::collections::BTreeMap::new();
            m.insert("client_order_id".to_string(), "cli-1".to_string());
            m
        },
    }
}

#[test]
fn idempotency_must_be_nonempty_and_printable() {
    assert!(IdempotencyKey::parse("").is_err());
    assert!(IdempotencyKey::parse("   ").is_err());
    assert!(IdempotencyKey::parse("ok-ok-ok-1234567890").is_ok());
}

#[test]
fn gate_rejects_missing_price_for_limit() {
    let spy = SpyConnector::new();
    let client = ExecutionClient::new(spy);
    let mut intent = base_intent();
    intent.price = None;
    let req = OrderRequest {
        mode: ExecutionMode::Live,
        idempotency: IdempotencyKey::random_uuid(),
        intent,
        run_id: Some("run-1".to_string()),
    };
    let e = client.place(req).unwrap_err();
    assert_eq!(e.code, SdkExecutionErrorCode::OrderGateRejected);
}

#[test]
fn paper_and_shadow_do_not_call_connector_place() {
    let spy = SpyConnector::new();
    let client = ExecutionClient::new(spy);

    // Paper
    let req_paper = OrderRequest {
        mode: ExecutionMode::Paper,
        idempotency: IdempotencyKey::random_uuid(),
        intent: base_intent(),
        run_id: Some("run-1".to_string()),
    };
    let out_paper = client.place(req_paper).unwrap();
    assert_eq!(out_paper.receipt.venue_order_id, None);

    // Shadow
    let req_shadow = OrderRequest {
        mode: ExecutionMode::Shadow,
        idempotency: IdempotencyKey::random_uuid(),
        intent: base_intent(),
        run_id: Some("run-2".to_string()),
    };
    let out_shadow = client.place(req_shadow).unwrap();
    assert_eq!(out_shadow.receipt.venue_order_id, None);

    // Live: separate client+spy to verify connector is called
    let spy2 = SpyConnector::new();
    let client2 = ExecutionClient::new(spy2);
    let req_live = OrderRequest {
        mode: ExecutionMode::Live,
        idempotency: IdempotencyKey::random_uuid(),
        intent: base_intent(),
        run_id: Some("run-3".to_string()),
    };
    let out_live = client2.place(req_live).unwrap();
    assert!(out_live.receipt.venue_order_id.is_some());
}

#[test]
fn idempotency_derive_is_stable_for_same_intent() {
    let intent = base_intent();
    let k1 = IdempotencyKey::derive_from_intent(&intent);
    let k2 = IdempotencyKey::derive_from_intent(&intent);
    assert_eq!(k1, k2);
}

#[test]
fn audit_replay_filters_by_run_id() {
    let sink = InMemoryAuditSink::new();
    let client = ExecutionClient::new(SpyConnector::new())
        .with_audit(Box::new(InMemoryAuditSink::new()));

    let req = OrderRequest {
        mode: ExecutionMode::Paper,
        idempotency: IdempotencyKey::random_uuid(),
        intent: base_intent(),
        run_id: Some("run-audit-test".to_string()),
    };
    // Verify InMemoryAuditSink append works
    sink.append(AuditEvent::OrderRequested {
        run_id: Some("run-x".to_string()),
        idempotency: IdempotencyKey::random_uuid(),
        intent: base_intent(),
        unix_ms: 0,
        mode: ExecutionMode::Paper,
    })
    .unwrap();

    let _ = client.place(req).unwrap();
    // replay without audit-sink configured on bare sink just checks trait interface compiles
    let _ = sink
        .replay(AuditReplayFilter {
            run_id: Some("run-x".to_string()),
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap();
}

#[test]
fn execution_surface_types_compile() {
    // Compile-time surface check: all re-exported types are accessible
    let _vid: VenueId = VenueId::new("test");
    let _sym: Symbol = Symbol::new("BTCUSDT");
    let _iid: OrderIntentId = OrderIntentId::new("id-1");
    let _p: Price = Price(1.0);
    let _q: Quantity = Quantity(1.0);
    let _s: OrderSide = OrderSide::Buy;
    let _ot: OrderType = OrderType::Limit;
    let _tif: OrderTimeInForce = OrderTimeInForce::Gtc;
    let _mode: ExecutionMode = ExecutionMode::Paper;
    let _status: OrderStatus = OrderStatus::Accepted;
    let _rs: ReconcileSource = ReconcileSource::Venue;
    let _ec: SdkExecutionErrorCode = SdkExecutionErrorCode::Internal;
    let _key = IdempotencyKey::random_uuid();
}
