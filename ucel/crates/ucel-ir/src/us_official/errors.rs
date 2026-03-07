use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsOfficialErrorCode {
    PolicyBlocked,
    ReviewRequired,
    IssuerNotFound,
    AmbiguousIssuer,
    DocumentNotFound,
    ArtifactNotFound,
    UnsupportedArtifactKind,
    OversizedArtifact,
    InvalidContentType,
    ParseFailed,
    SourceUnavailable,
    InvalidFilingFamilyMapping,
}

#[derive(Debug, Error)]
#[error("{code:?}: {message}")]
pub struct UsOfficialError {
    pub code: UsOfficialErrorCode,
    pub message: String,
}

impl UsOfficialError {
    pub fn new(code: UsOfficialErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
