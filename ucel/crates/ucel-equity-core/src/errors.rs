use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum EquityAdapterErrorKind {
    #[error("vendor timeout")]
    VendorTimeout,
    #[error("rate limited")]
    RateLimited,
    #[error("unauthorized")]
    Unauthorized,
    #[error("unsupported symbol")]
    UnsupportedSymbol,
    #[error("delayed feed only")]
    DelayedOnly,
    #[error("malformed response")]
    MalformedResponse,
    #[error("calendar unavailable")]
    CalendarUnavailable,
    #[error("corporate action unavailable")]
    CorporateActionUnavailable,
    #[error("ambiguous symbol mapping")]
    AmbiguousSymbol,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{kind}: {message}")]
pub struct EquityAdapterError {
    pub kind: EquityAdapterErrorKind,
    pub message: String,
}

impl EquityAdapterError {
    pub fn new(kind: EquityAdapterErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}
