use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type SdkExecutionResult<T> = Result<T, SdkExecutionError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdkExecutionErrorCode {
    InvalidInput,
    OrderGateRejected,
    NotSupported,
    ConnectorError,
    IdempotencyViolation,
    AuditFailure,
    ReconcileFailure,
    ReplayFailure,
    Timeout,
    Internal,
}

#[derive(Error, Debug)]
#[error("{message} [{code:?}]")]
pub struct SdkExecutionError {
    pub code: SdkExecutionErrorCode,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl SdkExecutionError {
    pub fn new(code: SdkExecutionErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source<E>(mut self, e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(e));
        self
    }
}
