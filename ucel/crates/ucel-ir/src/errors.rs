use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UcelIrErrorKind {
    Config,
    Http,
    Upstream,
    RateLimit,
    Sink,
    Checkpoint,
    NotImplemented,
    Internal,
}

#[derive(Debug, Error)]
#[error("{kind:?}: {message}")]
pub struct UcelIrError {
    pub kind: UcelIrErrorKind,
    pub message: String,
}

impl UcelIrError {
    pub fn new(kind: UcelIrErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            UcelIrErrorKind::Http | UcelIrErrorKind::Upstream | UcelIrErrorKind::RateLimit
        )
    }
}
