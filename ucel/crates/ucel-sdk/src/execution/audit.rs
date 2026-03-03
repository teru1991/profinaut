use crate::execution::{SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuditEvent {
    OrderRequested {
        run_id: Option<String>,
        idempotency: crate::execution::IdempotencyKey,
        intent: crate::execution::OrderIntent,
        unix_ms: u64,
        mode: crate::execution::ExecutionMode,
    },
    OrderResult {
        run_id: Option<String>,
        idempotency: crate::execution::IdempotencyKey,
        intent_id: crate::execution::OrderIntentId,
        receipt: crate::execution::OrderReceipt,
        unix_ms: u64,
    },
    CancelRequested {
        run_id: Option<String>,
        idempotency: crate::execution::IdempotencyKey,
        venue_order_id: String,
        venue: crate::execution::VenueId,
        symbol: crate::execution::Symbol,
        unix_ms: u64,
    },
    CancelResult {
        run_id: Option<String>,
        idempotency: crate::execution::IdempotencyKey,
        venue_order_id: String,
        ok: bool,
        unix_ms: u64,
    },
    ReconcileResult {
        venue: crate::execution::VenueId,
        report: crate::execution::ReconcileReport,
    },
}

/// AuditSink は「監査の唯一の差し込み口」
/// - 実装は次タスクで file/WAL/remote に拡張できるよう trait で固定する
pub trait AuditSink: Send + Sync {
    fn append(&self, event: AuditEvent) -> SdkExecutionResult<Option<String>>;
    fn replay(&self, filter: AuditReplayFilter) -> SdkExecutionResult<Vec<AuditEvent>>;
}

#[derive(Clone, Debug)]
pub struct AuditReplayFilter {
    pub run_id: Option<String>,
    pub venue: Option<crate::execution::VenueId>,
    pub intent_id: Option<crate::execution::OrderIntentId>,
    pub idempotency: Option<crate::execution::IdempotencyKey>,
    pub since_unix_ms: Option<u64>,
    pub until_unix_ms: Option<u64>,
}

/// テスト用の in-memory 実装（本番は次タスクで WAL/永続化へ）
pub struct InMemoryAuditSink {
    events: std::sync::Mutex<Vec<(Option<String>, AuditEvent)>>,
}

impl Default for InMemoryAuditSink {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryAuditSink {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl AuditSink for InMemoryAuditSink {
    fn append(&self, event: AuditEvent) -> SdkExecutionResult<Option<String>> {
        let mut g = self.events.lock().map_err(|_| {
            SdkExecutionError::new(SdkExecutionErrorCode::AuditFailure, "audit lock poisoned")
        })?;
        // 監査イベントIDは最小実装として index を文字列化
        let id = g.len().to_string();
        g.push((Some(id.clone()), event));
        Ok(Some(id))
    }

    fn replay(&self, filter: AuditReplayFilter) -> SdkExecutionResult<Vec<AuditEvent>> {
        let g = self.events.lock().map_err(|_| {
            SdkExecutionError::new(SdkExecutionErrorCode::ReplayFailure, "audit lock poisoned")
        })?;
        let mut out = Vec::new();
        for (_id, ev) in g.iter() {
            if let Some(ref run_id) = filter.run_id {
                let ok = match ev {
                    AuditEvent::OrderRequested { run_id: r, .. } => r.as_deref() == Some(run_id),
                    AuditEvent::OrderResult { run_id: r, .. } => r.as_deref() == Some(run_id),
                    AuditEvent::CancelRequested { run_id: r, .. } => {
                        r.as_deref() == Some(run_id)
                    }
                    AuditEvent::CancelResult { run_id: r, .. } => r.as_deref() == Some(run_id),
                    AuditEvent::ReconcileResult { .. } => false,
                };
                if !ok {
                    continue;
                }
            }
            // venue / intent_id / idempotency は必要になったら拡張（今は public surface 固定を優先）
            out.push(ev.clone());
        }
        Ok(out)
    }
}
