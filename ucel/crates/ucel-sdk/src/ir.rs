use crate::error::SdkResult;
use ucel_core::{IrAccessPolicyClass, IrArtifactDescriptor, IrDocumentDescriptor};
use ucel_ir::{
    IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse,
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse, IrIssuerResolutionInput, IrIssuerResolutionResult,
};
use ucel_registry::hub::registry;

#[derive(Debug, Default, Clone)]
pub struct IrFacade;

impl IrFacade {
    pub fn list_ir_sources(&self) -> SdkResult<Vec<registry::IrInventorySource>> {
        Ok(registry::list_ir_sources()?)
    }

    pub fn resolve_ir_issuer(
        &self,
        _input: &IrIssuerResolutionInput,
    ) -> SdkResult<Option<IrIssuerResolutionResult>> {
        Ok(None)
    }

    pub fn list_ir_documents(
        &self,
        _request: &IrDocumentListRequest,
    ) -> SdkResult<IrDocumentListResponse> {
        Ok(IrDocumentListResponse {
            documents: Vec::<IrDocumentDescriptor>::new(),
            next_cursor: None,
        })
    }

    pub fn fetch_ir_document_detail(
        &self,
        request: &IrDocumentDetailRequest,
    ) -> SdkResult<IrDocumentDetailResponse> {
        Err(crate::error::SdkError::Config(format!(
            "document detail is not implemented for source={} doc={}",
            request.source_id, request.key.source_document_id
        )))
    }

    pub fn list_ir_artifacts(
        &self,
        _request: &IrArtifactListRequest,
    ) -> SdkResult<IrArtifactListResponse> {
        Ok(IrArtifactListResponse {
            artifacts: Vec::<IrArtifactDescriptor>::new(),
        })
    }

    pub fn fetch_ir_artifact(
        &self,
        request: &IrArtifactFetchRequest,
    ) -> SdkResult<IrArtifactFetchResponse> {
        Err(crate::error::SdkError::Config(format!(
            "artifact fetch is not implemented for source={} artifact={}",
            request.source_id, request.artifact_key.artifact_id
        )))
    }

    pub fn preview_ir_source_support(&self) -> SdkResult<Vec<(String, IrAccessPolicyClass)>> {
        Ok(self
            .list_ir_sources()?
            .into_iter()
            .map(|s| {
                let policy = match s.access_policy_class.as_str() {
                    "free_public_noauth_allowed" => IrAccessPolicyClass::FreePublicNoAuthAllowed,
                    "free_public_noauth_review_required" => {
                        IrAccessPolicyClass::FreePublicNoAuthReviewRequired
                    }
                    "excluded_paid_or_contract" => IrAccessPolicyClass::ExcludedPaidOrContract,
                    "excluded_login_required" => IrAccessPolicyClass::ExcludedLoginRequired,
                    _ => IrAccessPolicyClass::ExcludedPolicyBlocked,
                };
                (s.source_id, policy)
            })
            .collect())
    }
}
