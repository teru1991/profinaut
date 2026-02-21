use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    FilingDocument,
    FilingMetadata,
    FilingXbrl,
    Attachment,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub kind: ArtifactKind,
    pub uri: String,
    pub source_url: String,
    pub sha256: Option<String>,
    pub content_length: Option<u64>,
    pub mime: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub retrieved_at: Option<u64>,
}
