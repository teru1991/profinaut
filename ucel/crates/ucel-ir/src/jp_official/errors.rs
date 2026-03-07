use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpOfficialErrorCode {
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
}

#[derive(Debug, Error)]
#[error("{code:?}: {message}")]
pub struct JpOfficialError {
    pub code: JpOfficialErrorCode,
    pub message: String,
}

impl JpOfficialError {
    pub fn new(code: JpOfficialErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
