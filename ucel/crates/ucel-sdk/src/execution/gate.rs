use crate::execution::{SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult};

/// OrderGate は "入口" で必ず適用されるガード。
/// 実際の tick/step/min_notional 等の強い検証は UCEL の MarketMeta と連携して次タスクで強化できるが、
/// このタスクでは public surface を固定し、最低限の事故防止（無効値拒否）を保証する。
pub trait OrderGate: Send + Sync {
    fn validate(&self, req: &crate::execution::OrderRequest) -> SdkExecutionResult<()>;
}

/// 最小実装：intent の basic validate を適用
pub struct BasicOrderGate;

impl OrderGate for BasicOrderGate {
    fn validate(&self, req: &crate::execution::OrderRequest) -> SdkExecutionResult<()> {
        req.intent.validate_basic().map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::OrderGateRejected,
                format!("order gate rejected: {e}"),
            )
        })?;
        Ok(())
    }
}
