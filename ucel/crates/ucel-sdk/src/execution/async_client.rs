use crate::execution::{
    AuditEvent, AuditReplayFilter, AuditSink, BasicOrderGate, ExecutionMode, ExecutionOutcome,
    OrderCancel, OrderGate, OrderOpenQuery, OrderReceipt, OrderRequest, OrderStatus,
    ReconcileReport, SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult, VenueId,
};

pub fn unix_ms_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 本番向け async 実行 trait。venue 実装が満たすべき契約（非同期版）。
/// sync の ExecutionConnector と並立し、互換を壊さない。
#[allow(async_fn_in_trait)]
pub trait ExecutionConnectorAsync: Send + Sync {
    async fn place_order(&self, req: &OrderRequest) -> SdkExecutionResult<OrderReceipt>;
    async fn cancel_order(&self, cancel: &OrderCancel) -> SdkExecutionResult<bool>;
    async fn list_open_orders(&self, q: &OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>>;
    async fn reconcile(&self, _venue: &VenueId) -> SdkExecutionResult<ReconcileReport> {
        Err(SdkExecutionError::new(
            SdkExecutionErrorCode::NotSupported,
            "reconcile not supported",
        ))
    }
}

/// ucel-sdk の "唯一の async 発注出口"。
/// - sync の ExecutionClient は互換維持し残す（非推奨へ）
/// - 本番用途はこの ExecutionClientAsync を推奨
pub struct ExecutionClientAsync<C: ExecutionConnectorAsync> {
    connector: C,
    gate: Box<dyn OrderGate>,
    audit: Option<Box<dyn AuditSink>>,
}

impl<C: ExecutionConnectorAsync> ExecutionClientAsync<C> {
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

    pub async fn place(&self, mut req: OrderRequest) -> SdkExecutionResult<ExecutionOutcome> {
        // Gate（入口の共通検証）
        self.gate.validate(&req)?;

        // Live で client_order_id が未指定なら idempotency から注入（事故防止）
        if matches!(req.mode, ExecutionMode::Live)
            && !req.intent.tags.contains_key("client_order_id")
        {
            req.intent
                .tags
                .insert("client_order_id".to_string(), req.idempotency.0.clone());
        }

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

        let receipt = match req.mode {
            ExecutionMode::Paper | ExecutionMode::Shadow => OrderReceipt {
                venue: req.intent.venue.clone(),
                symbol: req.intent.symbol.clone(),
                status: OrderStatus::Accepted,
                venue_order_id: None,
                client_order_id: req.intent.tags.get("client_order_id").cloned(),
                intent_id: req.intent.intent_id.clone(),
                idempotency: req.idempotency.clone(),
            },
            ExecutionMode::Live => self.connector.place_order(&req).await?,
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

    pub async fn cancel(&self, cancel: OrderCancel) -> SdkExecutionResult<bool> {
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
        let ok = self.connector.cancel_order(&cancel).await?;
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

    pub async fn open_orders(&self, q: OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>> {
        self.connector.list_open_orders(&q).await
    }

    pub async fn reconcile(&self, venue: &VenueId) -> SdkExecutionResult<ReconcileReport> {
        let mut r = self.connector.reconcile(venue).await?;
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
