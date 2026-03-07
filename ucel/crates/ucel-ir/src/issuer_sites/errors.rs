use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssuerSiteErrorCode {
    PolicyBlocked,
    ReviewRequired,
    IssuerSiteNotFound,
    AmbiguousSiteBinding,
    SectionNotFound,
    DocumentNotFound,
    ArtifactNotFound,
    UnsupportedArtifactKind,
    OversizedArtifact,
    InvalidContentType,
    ParseFailed,
    DiscoveryBudgetExceeded,
    CrawlDepthExceeded,
    SourceUnavailable,
}

#[derive(Debug, Error)]
#[error("{code:?}: {message}")]
pub struct IssuerSiteError {
    pub code: IssuerSiteErrorCode,
    pub message: String,
}

impl IssuerSiteError {
    pub fn new(code: IssuerSiteErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
