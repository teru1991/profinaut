use crate::error::{SdkError, SdkResult};
use ucel_core::{
    IrAccessPolicyClass, IrArtifactDescriptor, IrDocumentDescriptor, IrIssuerIdentityKind,
    IrIssuerKey, IrMarket, IrNormalizedContent,
};
use ucel_ir::{
    jp_issuer_feed_adapter, jp_issuer_html_adapter, statutory_adapter, timely_adapter,
    us_issuer_feed_adapter, us_issuer_html_adapter, normalize_artifact, IrArtifactFetchRequest, IrArtifactFetchResponse,
    IrArtifactListRequest, IrArtifactListResponse, IrDiscoverIssuersRequest,
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse, IrIssuerResolutionInput, IrIssuerResolutionResult, IrSourceAdapter,
};
use ucel_registry::hub::registry;

#[derive(Debug, Default, Clone)]
pub struct IrFacade;

impl IrFacade {
    fn adapter_for(source_id: &str) -> SdkResult<Box<dyn IrSourceAdapter>> {
        match source_id {
            "edinet_api_documents_v2" => Ok(Box::new(statutory_adapter())),
            "jp_tdnet_timely_html" => Ok(Box::new(timely_adapter())),
            "jp_issuer_ir_html_public" => Ok(Box::new(jp_issuer_html_adapter())),
            "jp_issuer_ir_feed_public" => Ok(Box::new(jp_issuer_feed_adapter())),
            "us_issuer_ir_html_public" => Ok(Box::new(us_issuer_html_adapter())),
            "us_issuer_ir_feed_public" => Ok(Box::new(us_issuer_feed_adapter())),
            _ => Err(SdkError::Config(format!(
                "unsupported source route for sdk facade: {source_id}"
            ))),
        }
    }

    pub fn list_ir_sources(&self) -> SdkResult<Vec<registry::IrInventorySource>> {
        Ok(registry::list_ir_sources()?)
    }

    pub fn resolve_ir_issuer(
        &self,
        input: &IrIssuerResolutionInput,
    ) -> SdkResult<Option<IrIssuerResolutionResult>> {
        let adapter = Self::adapter_for(&input.source_id)?;
        Ok(Some(
            adapter
                .resolve_issuer(input)
                .map_err(|e| SdkError::Config(e.to_string()))?,
        ))
    }

    pub fn list_ir_documents(
        &self,
        request: &IrDocumentListRequest,
    ) -> SdkResult<IrDocumentListResponse> {
        let adapter = Self::adapter_for(&request.source_id)?;
        Ok(adapter
            .list_documents(request)
            .map_err(|e| SdkError::Config(e.to_string()))?)
    }

    pub fn fetch_ir_document_detail(
        &self,
        request: &IrDocumentDetailRequest,
    ) -> SdkResult<IrDocumentDetailResponse> {
        let adapter = Self::adapter_for(&request.source_id)?;
        Ok(adapter
            .fetch_document_detail(request)
            .map_err(|e| SdkError::Config(e.to_string()))?)
    }

    pub fn list_ir_artifacts(
        &self,
        request: &IrArtifactListRequest,
    ) -> SdkResult<IrArtifactListResponse> {
        let adapter = Self::adapter_for(&request.source_id)?;
        Ok(adapter
            .list_artifacts(request)
            .map_err(|e| SdkError::Config(e.to_string()))?)
    }

    pub fn fetch_ir_artifact(
        &self,
        request: &IrArtifactFetchRequest,
    ) -> SdkResult<IrArtifactFetchResponse> {
        let adapter = Self::adapter_for(&request.source_id)?;
        Ok(adapter
            .fetch_artifact(request)
            .map_err(|e| SdkError::Config(e.to_string()))?)
    }


    pub fn normalize_ir_artifact(&self, request: &IrArtifactFetchRequest) -> SdkResult<IrNormalizedContent> {
        let fetched = self.fetch_ir_artifact(request)?;
        normalize_artifact(&fetched).map_err(|e| SdkError::Config(e.message))
    }

    pub fn preview_ir_normalization_support(&self) -> SdkResult<Vec<String>> {
        Ok(registry::list_ir_normalizable_formats())
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

    pub fn discover_jp_official_issuers(
        &self,
        source_id: &str,
        query: Option<String>,
    ) -> SdkResult<Vec<IrIssuerResolutionResult>> {
        let adapter = Self::adapter_for(source_id)?;
        let res = adapter
            .discover_issuers(&IrDiscoverIssuersRequest {
                source_id: source_id.to_string(),
                query,
            })
            .map_err(|e| SdkError::Config(e.to_string()))?;
        Ok(res.issuers)
    }

    pub fn preview_jp_official_document_summary(
        &self,
        source_id: &str,
    ) -> SdkResult<(usize, Vec<IrDocumentDescriptor>, Vec<IrArtifactDescriptor>)> {
        let docs = self
            .list_ir_documents(&IrDocumentListRequest {
                source_id: source_id.to_string(),
                market: IrMarket::Jp,
                issuer_key: None,
            })
            .map_err(|e| SdkError::Config(e.to_string()))?;
        let mut artifacts = Vec::new();
        for doc in &docs.documents {
            let listed = self
                .list_ir_artifacts(&IrArtifactListRequest {
                    source_id: source_id.to_string(),
                    document_key: doc.key.clone(),
                })
                .map_err(|e| SdkError::Config(e.to_string()))?;
            artifacts.extend(listed.artifacts);
        }
        Ok((docs.documents.len(), docs.documents, artifacts))
    }


    pub fn preview_issuer_site_document_summary(
        &self,
        source_id: &str,
        market: IrMarket,
    ) -> SdkResult<(usize, Vec<IrDocumentDescriptor>, Vec<IrArtifactDescriptor>)> {
        let docs = self
            .list_ir_documents(&IrDocumentListRequest {
                source_id: source_id.to_string(),
                market,
                issuer_key: None,
            })
            .map_err(|e| SdkError::Config(e.to_string()))?;
        let mut artifacts = Vec::new();
        for doc in &docs.documents {
            let listed = self
                .list_ir_artifacts(&IrArtifactListRequest {
                    source_id: source_id.to_string(),
                    document_key: doc.key.clone(),
                })
                .map_err(|e| SdkError::Config(e.to_string()))?;
            artifacts.extend(listed.artifacts);
        }
        Ok((docs.documents.len(), docs.documents, artifacts))
    }

    pub fn resolve_jp_by_code(
        &self,
        source_id: &str,
        code: &str,
    ) -> SdkResult<Option<IrIssuerKey>> {
        let out = self
            .resolve_ir_issuer(&IrIssuerResolutionInput {
                market: IrMarket::Jp,
                source_id: source_id.to_string(),
                identity_kind: IrIssuerIdentityKind::JpExchangeCodeLike,
                value: code.to_string(),
            })
            .map_err(|e| SdkError::Config(e.to_string()))?;
        Ok(out.map(|x| x.issuer_key))
    }
}
