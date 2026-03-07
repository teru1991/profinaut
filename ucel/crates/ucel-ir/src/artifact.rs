use ucel_core::{IrArtifactDescriptor, IrArtifactKey, IrDocumentKey};

#[derive(Debug, Clone)]
pub struct IrArtifactListRequest {
    pub source_id: String,
    pub document_key: IrDocumentKey,
}

#[derive(Debug, Clone)]
pub struct IrArtifactListResponse {
    pub artifacts: Vec<IrArtifactDescriptor>,
}

#[derive(Debug, Clone)]
pub struct IrArtifactFetchRequest {
    pub source_id: String,
    pub artifact_key: IrArtifactKey,
}

#[derive(Debug, Clone)]
pub struct IrArtifactFetchResponse {
    pub metadata: IrArtifactDescriptor,
    pub bytes: Option<Vec<u8>>,
    pub text_candidate: Option<String>,
    pub source_metadata: serde_json::Value,
}
