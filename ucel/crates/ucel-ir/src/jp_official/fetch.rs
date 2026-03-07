use super::access::{ensure_policy_allowed, JpPolitenessPolicy};
use super::{artifact, document, identity};
use crate::artifact::{
    IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse,
};
use crate::document::{
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse,
};
use crate::errors::{UcelIrError, UcelIrErrorKind};
use crate::fetch::{
    IrDiscoverIssuersRequest, IrDiscoverIssuersResponse, IrFetchMode, IrSourceAdapter,
};
use crate::identity::{IrIssuerResolutionInput, IrIssuerResolutionResult};
use ucel_core::{
    IrAccessPattern, IrAccessPolicyClass, IrMarket, IrSourceDescriptor, IrSourceFamily,
    IrSourceKind,
};

#[derive(Debug, Clone)]
pub struct JpOfficialAdapter {
    source_id: String,
    policy: JpPolitenessPolicy,
}

impl JpOfficialAdapter {
    pub fn new(source_id: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            policy: JpPolitenessPolicy::default(),
        }
    }

    fn descriptor_for_source(&self) -> IrSourceDescriptor {
        match self.source_id.as_str() {
            "edinet_api_documents_v2" => IrSourceDescriptor {
                source_id: self.source_id.clone(),
                market: IrMarket::Jp,
                source_family: IrSourceFamily::JpStatutoryDisclosure,
                source_kind: IrSourceKind::OfficialPublicApi,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthAllowed,
                access_patterns: vec![
                    IrAccessPattern::IssuerLookup,
                    IrAccessPattern::ApiList,
                    IrAccessPattern::ApiDetail,
                    IrAccessPattern::ArtifactDownload,
                ],
            },
            _ => IrSourceDescriptor {
                source_id: self.source_id.clone(),
                market: IrMarket::Jp,
                source_family: IrSourceFamily::JpTimelyDisclosure,
                source_kind: IrSourceKind::OfficialPublicHtml,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
                access_patterns: vec![
                    IrAccessPattern::IssuerLookup,
                    IrAccessPattern::HtmlIndex,
                    IrAccessPattern::HtmlDetail,
                    IrAccessPattern::FeedPoll,
                    IrAccessPattern::ArtifactDownload,
                ],
            },
        }
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }
}

impl IrSourceAdapter for JpOfficialAdapter {
    fn source_descriptor(&self) -> IrSourceDescriptor {
        self.descriptor_for_source()
    }

    fn discover_issuers(
        &self,
        request: &IrDiscoverIssuersRequest,
    ) -> Result<IrDiscoverIssuersResponse, UcelIrError> {
        let input = IrIssuerResolutionInput {
            market: IrMarket::Jp,
            source_id: request.source_id.clone(),
            identity_kind: ucel_core::IrIssuerIdentityKind::UrlLike,
            value: request.query.clone().unwrap_or_else(|| {
                if request.source_id == "edinet_api_documents_v2" {
                    "E12345".into()
                } else {
                    "5678".into()
                }
            }),
        };
        let issuer = identity::resolve(&input)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;
        Ok(IrDiscoverIssuersResponse {
            issuers: vec![issuer],
            mode: if request.source_id == "edinet_api_documents_v2" {
                IrFetchMode::Api
            } else {
                IrFetchMode::Html
            },
            metadata: serde_json::json!({"source_id": request.source_id}),
        })
    }

    fn resolve_issuer(
        &self,
        input: &IrIssuerResolutionInput,
    ) -> Result<IrIssuerResolutionResult, UcelIrError> {
        identity::resolve(input)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn list_documents(
        &self,
        request: &IrDocumentListRequest,
    ) -> Result<IrDocumentListResponse, UcelIrError> {
        document::list(request)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn fetch_document_detail(
        &self,
        request: &IrDocumentDetailRequest,
    ) -> Result<IrDocumentDetailResponse, UcelIrError> {
        document::detail(request)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn list_artifacts(
        &self,
        request: &IrArtifactListRequest,
    ) -> Result<IrArtifactListResponse, UcelIrError> {
        artifact::list(request)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn fetch_artifact(
        &self,
        request: &IrArtifactFetchRequest,
    ) -> Result<IrArtifactFetchResponse, UcelIrError> {
        let descriptor = self.descriptor_for_source();
        ensure_policy_allowed(descriptor.access_policy_class)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Policy, e.to_string()))?;
        artifact::fetch(request, self.policy)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }
}
