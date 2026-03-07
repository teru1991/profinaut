use crate::artifact::{
    IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse,
};
use crate::document::{
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse,
};
use crate::errors::UcelIrError;
use crate::identity::{IrIssuerResolutionInput, IrIssuerResolutionResult};
use ucel_core::IrSourceDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrFetchMode {
    Api,
    Feed,
    Html,
    Attachment,
}

#[derive(Debug, Clone)]
pub struct IrDiscoverIssuersRequest {
    pub source_id: String,
    pub query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IrDiscoverIssuersResponse {
    pub issuers: Vec<IrIssuerResolutionResult>,
    pub mode: IrFetchMode,
    pub metadata: serde_json::Value,
}

pub trait IrSourceAdapter {
    fn source_descriptor(&self) -> IrSourceDescriptor;
    fn discover_issuers(
        &self,
        request: &IrDiscoverIssuersRequest,
    ) -> Result<IrDiscoverIssuersResponse, UcelIrError>;
    fn resolve_issuer(
        &self,
        input: &IrIssuerResolutionInput,
    ) -> Result<IrIssuerResolutionResult, UcelIrError>;
    fn list_documents(
        &self,
        request: &IrDocumentListRequest,
    ) -> Result<IrDocumentListResponse, UcelIrError>;
    fn fetch_document_detail(
        &self,
        request: &IrDocumentDetailRequest,
    ) -> Result<IrDocumentDetailResponse, UcelIrError>;
    fn list_artifacts(
        &self,
        request: &IrArtifactListRequest,
    ) -> Result<IrArtifactListResponse, UcelIrError>;
    fn fetch_artifact(
        &self,
        request: &IrArtifactFetchRequest,
    ) -> Result<IrArtifactFetchResponse, UcelIrError>;
}
