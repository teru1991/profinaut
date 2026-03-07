use super::access::{ensure_policy_allowed, IssuerSitePolitenessPolicy};
use super::{artifact, document, identity};
use crate::artifact::{IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse};
use crate::document::{IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest, IrDocumentListResponse};
use crate::errors::{UcelIrError, UcelIrErrorKind};
use crate::fetch::{IrDiscoverIssuersRequest, IrDiscoverIssuersResponse, IrFetchMode, IrSourceAdapter};
use crate::identity::{IrIssuerResolutionInput, IrIssuerResolutionResult};
use ucel_core::{IrAccessPattern, IrAccessPolicyClass, IrMarket, IrSourceDescriptor, IrSourceFamily, IrSourceKind};

#[derive(Debug, Clone)]
pub struct IssuerSiteAdapter {
    source_id: String,
    policy: IssuerSitePolitenessPolicy,
}

impl IssuerSiteAdapter {
    pub fn new(source_id: &str) -> Self {
        Self { source_id: source_id.to_string(), policy: IssuerSitePolitenessPolicy::default() }
    }

    fn descriptor_for_source(&self) -> IrSourceDescriptor {
        match self.source_id.as_str() {
            "jp_issuer_ir_html_public" => IrSourceDescriptor {
                source_id: self.source_id.clone(), market: IrMarket::Jp,
                source_family: IrSourceFamily::JpIssuerIrSite, source_kind: IrSourceKind::IssuerIrHtml,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
                access_patterns: vec![IrAccessPattern::IssuerLookup, IrAccessPattern::HtmlIndex, IrAccessPattern::HtmlDetail, IrAccessPattern::AttachmentDiscover, IrAccessPattern::ArtifactDownload],
            },
            "jp_issuer_ir_feed_public" => IrSourceDescriptor {
                source_id: self.source_id.clone(), market: IrMarket::Jp,
                source_family: IrSourceFamily::JpIssuerIrSite, source_kind: IrSourceKind::IssuerIrFeed,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
                access_patterns: vec![IrAccessPattern::IssuerLookup, IrAccessPattern::FeedPoll, IrAccessPattern::AttachmentDiscover, IrAccessPattern::ArtifactDownload],
            },
            "us_issuer_ir_html_public" => IrSourceDescriptor {
                source_id: self.source_id.clone(), market: IrMarket::Us,
                source_family: IrSourceFamily::UsIssuerIrSite, source_kind: IrSourceKind::IssuerIrHtml,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
                access_patterns: vec![IrAccessPattern::IssuerLookup, IrAccessPattern::HtmlIndex, IrAccessPattern::HtmlDetail, IrAccessPattern::AttachmentDiscover, IrAccessPattern::ArtifactDownload],
            },
            _ => IrSourceDescriptor {
                source_id: self.source_id.clone(), market: IrMarket::Us,
                source_family: IrSourceFamily::UsIssuerIrSite, source_kind: IrSourceKind::IssuerIrFeed,
                access_policy_class: IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
                access_patterns: vec![IrAccessPattern::IssuerLookup, IrAccessPattern::FeedPoll, IrAccessPattern::AttachmentDiscover, IrAccessPattern::ArtifactDownload],
            },
        }
    }
}

impl IrSourceAdapter for IssuerSiteAdapter {
    fn source_descriptor(&self) -> IrSourceDescriptor { self.descriptor_for_source() }

    fn discover_issuers(&self, request: &IrDiscoverIssuersRequest) -> Result<IrDiscoverIssuersResponse, UcelIrError> {
        let market = if request.source_id.starts_with("jp_") { IrMarket::Jp } else { IrMarket::Us };
        let input = IrIssuerResolutionInput {
            market,
            source_id: request.source_id.clone(),
            identity_kind: ucel_core::IrIssuerIdentityKind::UrlLike,
            value: request.query.clone().unwrap_or_else(|| {
                if request.source_id.contains("jp_") { "acme.co.jp/ir".into() } else { "acme.com/investors".into() }
            }),
        };
        let issuer = identity::resolve(&input).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;
        Ok(IrDiscoverIssuersResponse {
            issuers: vec![issuer],
            mode: if request.source_id.ends_with("feed_public") { IrFetchMode::Feed } else { IrFetchMode::Html },
            metadata: serde_json::json!({"source_id": request.source_id, "discovery": "official_metadata_seed|inventory_seed|deterministic_traversal"}),
        })
    }

    fn resolve_issuer(&self, input: &IrIssuerResolutionInput) -> Result<IrIssuerResolutionResult, UcelIrError> {
        identity::resolve(input).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn list_documents(&self, request: &IrDocumentListRequest) -> Result<IrDocumentListResponse, UcelIrError> {
        document::list(request).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn fetch_document_detail(&self, request: &IrDocumentDetailRequest) -> Result<IrDocumentDetailResponse, UcelIrError> {
        document::detail(request).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn list_artifacts(&self, request: &IrArtifactListRequest) -> Result<IrArtifactListResponse, UcelIrError> {
        artifact::list(request).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }

    fn fetch_artifact(&self, request: &IrArtifactFetchRequest) -> Result<IrArtifactFetchResponse, UcelIrError> {
        ensure_policy_allowed(self.descriptor_for_source().access_policy_class)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Policy, e.to_string()))?;
        artifact::fetch(request, self.policy).map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }
}
