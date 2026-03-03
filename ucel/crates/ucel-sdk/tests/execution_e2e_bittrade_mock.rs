/// E2E 相当テスト: ExecutionClientAsync（唯一の async 出口）+ FileAuditSink（永続監査）
/// の組み合わせを検証する。
///
/// bittrade_mock という名前の通り、ここでは BittradeExecutionConnector を模倣した
/// インライン MockBittradeConnector を使い、循環依存を避けつつ SDK の async flow を証明する。
///
/// 実際の BittradeExecutionConnector との統合テストは:
///   ucel-cex-bittrade/tests/execution_connector_contract.rs を参照
use std::sync::Mutex;
use tempfile::tempdir;
use ucel_sdk::execution::*;

// ---------------------------------------------------------------------------
// インライン MockBittradeConnector
// ---------------------------------------------------------------------------

struct MockBittradeConnector {
    place_calls: Mutex<Vec<String>>,
    cancel_calls: Mutex<Vec<String>>,
}

impl MockBittradeConnector {
    fn new() -> Self {
        Self {
            place_calls: Mutex::new(vec![]),
            cancel_calls: Mutex::new(vec![]),
        }
    }
}

impl ExecutionConnectorAsync for MockBittradeConnector {
    async fn place_order(&self, req: &OrderRequest) -> SdkExecutionResult<OrderReceipt> {
        let cid = req
            .intent
            .tags
            .get("client_order_id")
            .cloned()
            .unwrap_or_default();
        self.place_calls.lock().unwrap().push(cid.clone());
        Ok(OrderReceipt {
            venue: req.intent.venue.clone(),
            symbol: req.intent.symbol.clone(),
            status: OrderStatus::Accepted,
            venue_order_id: Some("mock-venue-999".to_string()),
            client_order_id: Some(cid),
            intent_id: req.intent.intent_id.clone(),
            idempotency: req.idempotency.clone(),
        })
    }

    async fn cancel_order(&self, cancel: &OrderCancel) -> SdkExecutionResult<bool> {
        self.cancel_calls
            .lock()
            .unwrap()
            .push(cancel.venue_order_id.clone());
        Ok(true)
    }

    async fn list_open_orders(&self, _q: &OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>> {
        Ok(vec![])
    }

    async fn reconcile(&self, venue: &VenueId) -> SdkExecutionResult<ReconcileReport> {
        Ok(ReconcileReport {
            venue: venue.clone(),
            source: ReconcileSource::Venue,
            ok: true,
            mismatches: vec![],
            generated_at_unix_ms: unix_ms_now(),
        })
    }
}

fn mk_intent() -> OrderIntent {
    OrderIntent {
        intent_id: OrderIntentId::new("intent-e2e"),
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

// ---------------------------------------------------------------------------
// E2E: place + cancel → FileAuditSink → replay
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_async_e2e_with_file_audit() {
    let dir = tempdir().unwrap();
    let audit_path = dir.path().join("audit.ndjson");

    let audit = FileAuditSink::new(FileAuditSinkConfig {
        path: audit_path.clone(),
        fsync_each_append: false,
        max_line_bytes: 1024 * 1024,
    });

    let connector = MockBittradeConnector::new();
    let client = ExecutionClientAsync::new(connector).with_audit(Box::new(audit));

    let req = OrderRequest {
        mode: ExecutionMode::Live,
        intent: mk_intent(),
        idempotency: IdempotencyKey::parse("idem-e2e-1234567890abcdef").unwrap(),
        run_id: Some("run-e2e".to_string()),
    };

    // place（Live で client_order_id が自動注入される）
    let out = client.place(req).await.unwrap();
    assert_eq!(
        out.receipt.venue_order_id.as_deref(),
        Some("mock-venue-999")
    );
    // idempotency が client_order_id として注入されたことを確認
    assert_eq!(
        out.receipt.client_order_id.as_deref(),
        Some("idem-e2e-1234567890abcdef")
    );

    // cancel
    let cancel = OrderCancel {
        venue: VenueId::new("bittrade"),
        symbol: Symbol::new("BTCJPY"),
        venue_order_id: out.receipt.venue_order_id.clone().unwrap(),
        idempotency: IdempotencyKey::parse("cancel-e2e-1234567890abcdef").unwrap(),
        run_id: Some("run-e2e".to_string()),
    };
    let ok = client.cancel(cancel).await.unwrap();
    assert!(ok);

    // replay: run_id でフィルタ
    let events = client
        .replay(AuditReplayFilter {
            run_id: Some("run-e2e".to_string()),
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap();

    // 最低 4 イベント: OrderRequested / OrderResult / CancelRequested / CancelResult
    assert!(
        events.len() >= 4,
        "expected >= 4 audit events, got {}",
        events.len()
    );

    // ファイルが実際に書かれていることを確認
    assert!(audit_path.exists(), "audit file should exist");
    let content = std::fs::read_to_string(&audit_path).unwrap();
    let lines: Vec<_> = content.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 4,
        "expected >= 4 lines in audit file, got {}",
        lines.len()
    );

    // 各行が有効な JSON であることを確認（salvage 検証）
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("audit line is not valid JSON: {line} | error: {e}"));
        assert!(v.is_object(), "audit line should be JSON object");
    }
}

// ---------------------------------------------------------------------------
// audit なし（audit sink 未設定）で replay がエラーを返すことを確認
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_async_replay_without_audit_returns_error() {
    let connector = MockBittradeConnector::new();
    let client = ExecutionClientAsync::new(connector); // audit 未設定

    let err = client
        .replay(AuditReplayFilter {
            run_id: Some("any".to_string()),
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap_err();
    assert_eq!(err.code, SdkExecutionErrorCode::ReplayFailure);
}

// ---------------------------------------------------------------------------
// Paper/Shadow モードは connector の place_order を呼ばない
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_async_paper_does_not_call_connector() {
    let connector = MockBittradeConnector::new();
    let client = ExecutionClientAsync::new(connector);

    let req = OrderRequest {
        mode: ExecutionMode::Paper,
        intent: mk_intent(),
        idempotency: IdempotencyKey::random_uuid(),
        run_id: None,
    };
    let out = client.place(req).await.unwrap();
    assert_eq!(out.receipt.venue_order_id, None);
    assert_eq!(out.receipt.status, OrderStatus::Accepted);
}

// ---------------------------------------------------------------------------
// Gate: Limit で price なし は拒否される
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_async_gate_rejects_limit_without_price() {
    let connector = MockBittradeConnector::new();
    let client = ExecutionClientAsync::new(connector);

    let mut intent = mk_intent();
    intent.price = None;

    let req = OrderRequest {
        mode: ExecutionMode::Live,
        intent,
        idempotency: IdempotencyKey::random_uuid(),
        run_id: None,
    };
    let err = client.place(req).await.unwrap_err();
    assert_eq!(err.code, SdkExecutionErrorCode::OrderGateRejected);
}

// ---------------------------------------------------------------------------
// FileAuditSink: run_id フィルタが正しく機能する
// ---------------------------------------------------------------------------

#[tokio::test]
async fn file_audit_sink_replay_filters_by_run_id() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("audit_filter.ndjson");

    let sink = FileAuditSink::new(FileAuditSinkConfig {
        path: path.clone(),
        fsync_each_append: false,
        max_line_bytes: 1024 * 1024,
    });

    // run-A のイベント
    sink.append(AuditEvent::OrderRequested {
        run_id: Some("run-A".to_string()),
        idempotency: IdempotencyKey::parse("idem-run-a-1234567890abcd").unwrap(),
        intent: mk_intent(),
        unix_ms: 1000,
        mode: ExecutionMode::Live,
    })
    .unwrap();

    // run-B のイベント
    sink.append(AuditEvent::OrderRequested {
        run_id: Some("run-B".to_string()),
        idempotency: IdempotencyKey::parse("idem-run-b-1234567890abcd").unwrap(),
        intent: mk_intent(),
        unix_ms: 2000,
        mode: ExecutionMode::Paper,
    })
    .unwrap();

    // run-A でフィルタ
    let events = sink
        .replay(AuditReplayFilter {
            run_id: Some("run-A".to_string()),
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap();
    assert_eq!(events.len(), 1, "expected 1 event for run-A");

    // フィルタなし（全件）
    let all = sink
        .replay(AuditReplayFilter {
            run_id: None,
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap();
    assert_eq!(all.len(), 2, "expected 2 total events");
}

// ---------------------------------------------------------------------------
// FileAuditSink: ファイルが存在しない場合、replay は空を返す
// ---------------------------------------------------------------------------

#[tokio::test]
async fn file_audit_sink_replay_empty_when_file_missing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nonexistent.ndjson");

    let sink = FileAuditSink::new(FileAuditSinkConfig {
        path,
        fsync_each_append: false,
        max_line_bytes: 1024 * 1024,
    });

    let events = sink
        .replay(AuditReplayFilter {
            run_id: None,
            venue: None,
            intent_id: None,
            idempotency: None,
            since_unix_ms: None,
            until_unix_ms: None,
        })
        .unwrap();
    assert!(events.is_empty());
}

// ---------------------------------------------------------------------------
// reconcile: ExecutionClientAsync が生成時刻を補完し audit に残す
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_async_reconcile_audited() {
    let dir = tempdir().unwrap();
    let audit_path = dir.path().join("reconcile_audit.ndjson");

    let audit = FileAuditSink::new(FileAuditSinkConfig {
        path: audit_path.clone(),
        fsync_each_append: false,
        max_line_bytes: 1024 * 1024,
    });

    let connector = MockBittradeConnector::new();
    let client = ExecutionClientAsync::new(connector).with_audit(Box::new(audit));

    let report = client.reconcile(&VenueId::new("bittrade")).await.unwrap();
    assert!(report.ok);
    assert!(report.generated_at_unix_ms > 0);

    // audit ファイルに ReconcileResult が書かれている
    let content = std::fs::read_to_string(&audit_path).unwrap();
    assert!(
        content.contains("ReconcileResult"),
        "audit should contain ReconcileResult"
    );
}
