use ucel_core::PrivateRestOperation;

use crate::auth::{ExecutionAuthContext, ExecutionCorrelation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPrivateRestBridge {
    pub operation: PrivateRestOperation,
    pub auth: ExecutionAuthContext,
    pub correlation: ExecutionCorrelation,
}

impl ExecutionPrivateRestBridge {
    pub fn retry_safety_hint(&self) -> ucel_core::RetrySafety {
        match self.operation {
            PrivateRestOperation::CancelOrder => ucel_core::RetrySafety::UnsafeToRetry,
            _ => ucel_core::RetrySafety::Unknown,
        }
    }

    pub fn correlation_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.auth.venue, self.correlation.request_id, self.correlation.sequence
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_is_unsafe_to_retry() {
        let bridge = ExecutionPrivateRestBridge {
            operation: PrivateRestOperation::CancelOrder,
            auth: ExecutionAuthContext {
                venue: "bitbank".into(),
                key_id: Some("k".into()),
                request_name: "cancel_order".into(),
            },
            correlation: ExecutionCorrelation {
                run_id: "run".into(),
                request_id: "req".into(),
                sequence: 1,
            },
        };
        assert_eq!(
            bridge.retry_safety_hint(),
            ucel_core::RetrySafety::UnsafeToRetry
        );
    }
}
