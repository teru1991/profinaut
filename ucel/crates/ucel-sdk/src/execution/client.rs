use crate::execution::{
    AuditEvent, AuditReplayFilter, AuditSink, BasicOrderGate, ExecutionMode, ExecutionOutcome,
    OrderCancel, OrderGate, OrderOpenQuery, OrderReceipt, OrderRequest, OrderStatus,
    ReconcileReport, SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult,
};

/// ExecutionConnector は venue 実装が満たすべき契約。
/// - このタスクでは "全venue実装" までやらない（次タスク）
/// - ただし trait を固定し、ucel-sdk 入口が唯一の発注出口であることを保証する
pub trait ExecutionConnector: Send + Sync {
    fn place_order(&self, req: &OrderRequest) -> SdkExecutionResult<OrderReceipt>;
    fn cancel_order(&self, cancel: &OrderCancel) -> SdkExecutionResult<bool>;
    fn list_open_orders(&self, q: &OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>>;
    /// reconcile は best-effort。未対応なら NotSupported を返す。
    fn reconcile(
        &self,
        _venue: &crate::execution::VenueId,
    ) -> SdkExecutionResult<ReconcileReport> {
        Err(SdkExecutionError::new(
            SdkExecutionErrorCode::NotSupported,
            "reconcile not supported",
        ))
    }
}

/// ucel-sdk の "唯一の発注出口"
/// - mode / gate / audit を一箇所に集約し、各呼び出しが必ず監査に残る
pub struct ExecutionClient<C: ExecutionConnector> {
    connector: C,
    gate: Box<dyn OrderGate>,
    audit: Option<Box<dyn AuditSink>>,
}

impl<C: ExecutionConnector> ExecutionClient<C> {
    pub fn new(connector: C) -> Self {
        Self {
            connector,
            gate: Box::new(BasicOrderGate),
            audit: None,
        }
    }

    pub fn with_gate(mut self, gate: Box<dyn OrderGate>) -> Self {
        self.gate = gate;
        self
    }

    pub fn with_audit(mut self, audit: Box<dyn AuditSink>) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn place(&self, req: OrderRequest) -> SdkExecutionResult<ExecutionOutcome> {
        // 入口の共通検証（事故防止）
        self.gate.validate(&req)?;

        let now = unix_ms_now();

        // 監査：要求
        let audit_id_req = if let Some(a) = self.audit.as_ref() {
            a.append(AuditEvent::OrderRequested {
                run_id: req.run_id.clone(),
                idempotency: req.idempotency.clone(),
                intent: req.intent.clone(),
                unix_ms: now,
                mode: req.mode,
            })?
        } else {
            None
        };

        // mode に応じて挙動を固定
        let receipt = match req.mode {
            ExecutionMode::Paper => {
                // 純シミュレーション：Accepted を返す（venue_order_id なし）
                OrderReceipt {
                    venue: req.intent.venue.clone(),
                    symbol: req.intent.symbol.clone(),
                    status: OrderStatus::Accepted,
                    venue_order_id: None,
                    client_order_id: req.intent.tags.get("client_order_id").cloned(),
                    intent_id: req.intent.intent_id.clone(),
                    idempotency: req.idempotency.clone(),
                }
            }
            ExecutionMode::Shadow => {
                // 現状は "place_order を呼ばず" 監査だけ残す。
                // 次タスクで quote/validate に接続して照合を強化する。
                OrderReceipt {
                    venue: req.intent.venue.clone(),
                    symbol: req.intent.symbol.clone(),
                    status: OrderStatus::Accepted,
                    venue_order_id: None,
                    client_order_id: req.intent.tags.get("client_order_id").cloned(),
                    intent_id: req.intent.intent_id.clone(),
                    idempotency: req.idempotency.clone(),
                }
            }
            ExecutionMode::Live => {
                // 実発注：connector に委譲
                self.connector.place_order(&req)?
            }
        };

        // 監査：結果
        let audit_id_res = if let Some(a) = self.audit.as_ref() {
            a.append(AuditEvent::OrderResult {
                run_id: req.run_id.clone(),
                idempotency: req.idempotency.clone(),
                intent_id: req.intent.intent_id.clone(),
                receipt: receipt.clone(),
                unix_ms: unix_ms_now(),
            })?
        } else {
            None
        };

        Ok(ExecutionOutcome {
            receipt,
            audit_event_id: audit_id_res.or(audit_id_req),
        })
    }

    pub fn cancel(&self, cancel: OrderCancel) -> SdkExecutionResult<bool> {
        let now = unix_ms_now();
        if let Some(a) = self.audit.as_ref() {
            a.append(AuditEvent::CancelRequested {
                run_id: cancel.run_id.clone(),
                idempotency: cancel.idempotency.clone(),
                venue_order_id: cancel.venue_order_id.clone(),
                venue: cancel.venue.clone(),
                symbol: cancel.symbol.clone(),
                unix_ms: now,
            })?;
        }
        // 実行（このタスクでは mode は cancel に入れず "live cancel" を想定）
        let ok = self.connector.cancel_order(&cancel)?;
        if let Some(a) = self.audit.as_ref() {
            a.append(AuditEvent::CancelResult {
                run_id: cancel.run_id.clone(),
                idempotency: cancel.idempotency.clone(),
                venue_order_id: cancel.venue_order_id.clone(),
                ok,
                unix_ms: unix_ms_now(),
            })?;
        }
        Ok(ok)
    }

    pub fn open_orders(&self, q: OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>> {
        self.connector.list_open_orders(&q)
    }

    /// reconcile（照合）
    pub fn reconcile(
        &self,
        venue: &crate::execution::VenueId,
    ) -> SdkExecutionResult<ReconcileReport> {
        let mut r = self.connector.reconcile(venue)?;
        // source は venue が返したものを尊重しつつ、最低限埋める
        if r.generated_at_unix_ms == 0 {
            r.generated_at_unix_ms = unix_ms_now();
        }
        if let Some(a) = self.audit.as_ref() {
            a.append(AuditEvent::ReconcileResult {
                venue: venue.clone(),
                report: r.clone(),
            })?;
        }
        Ok(r)
    }

    /// replay（監査ログの再生）
    pub fn replay(&self, filter: AuditReplayFilter) -> SdkExecutionResult<Vec<AuditEvent>> {
        let a = self.audit.as_ref().ok_or_else(|| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::ReplayFailure,
                "audit sink not configured",
            )
        })?;
        a.replay(filter)
    }
}

fn unix_ms_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    d.as_millis() as u64
}
