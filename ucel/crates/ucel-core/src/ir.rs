use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrMarket {
    Jp,
    Us,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum IrSourceFamily {
    JpStatutoryDisclosure,
    JpTimelyDisclosure,
    JpIssuerIrSite,
    UsSecDisclosure,
    UsIssuerIrSite,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrSourceKind {
    OfficialPublicApi,
    OfficialPublicFeed,
    OfficialPublicHtml,
    IssuerIrHtml,
    IssuerIrFeed,
    AttachmentDownload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrAccessPolicyClass {
    FreePublicNoAuthAllowed,
    FreePublicNoAuthReviewRequired,
    ExcludedPaidOrContract,
    ExcludedLoginRequired,
    ExcludedPolicyBlocked,
}

impl IrAccessPolicyClass {
    pub fn to_access_decision(self) -> IrAccessDecision {
        match self {
            IrAccessPolicyClass::FreePublicNoAuthAllowed => IrAccessDecision::Allowed,
            IrAccessPolicyClass::FreePublicNoAuthReviewRequired => IrAccessDecision::ReviewRequired,
            IrAccessPolicyClass::ExcludedPaidOrContract
            | IrAccessPolicyClass::ExcludedLoginRequired
            | IrAccessPolicyClass::ExcludedPolicyBlocked => IrAccessDecision::Blocked,
        }
    }

    pub fn is_allowable(self) -> bool {
        matches!(
            self,
            IrAccessPolicyClass::FreePublicNoAuthAllowed
                | IrAccessPolicyClass::FreePublicNoAuthReviewRequired
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrAccessPattern {
    ApiList,
    ApiDetail,
    RssPoll,
    FeedPoll,
    HtmlIndex,
    HtmlDetail,
    AttachmentDiscover,
    ArtifactDownload,
    IssuerLookup,
    SearchQuery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrIssuerIdentityKind {
    JpEdinetCodeLike,
    JpLocalCodeLike,
    JpExchangeCodeLike,
    UsCikLike,
    TickerLike,
    ExchangeTickerLike,
    IssuerSiteSlugLike,
    UrlLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrDocumentFamily {
    StatutoryAnnual,
    StatutoryQuarterly,
    StatutoryCurrent,
    TimelyDisclosure,
    EarningsRelease,
    EarningsPresentation,
    Transcript,
    PressRelease,
    Proxy,
    IntegratedReport,
    SustainabilityReport,
    FactSheet,
    GovernanceReport,
    MiscIrDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrArtifactKind {
    Html,
    Pdf,
    Xbrl,
    Ixbrl,
    Xml,
    Txt,
    Csv,
    Zip,
    Json,
    Rss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrIssuerKey {
    pub market: IrMarket,
    pub canonical_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrIssuerAlias {
    pub market: IrMarket,
    pub source_id: String,
    pub identity_kind: IrIssuerIdentityKind,
    pub value: String,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrSourceDescriptor {
    pub source_id: String,
    pub market: IrMarket,
    pub source_family: IrSourceFamily,
    pub source_kind: IrSourceKind,
    pub access_policy_class: IrAccessPolicyClass,
    pub access_patterns: Vec<IrAccessPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrDocumentKey {
    pub source_id: String,
    pub source_document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrDocumentDescriptor {
    pub key: IrDocumentKey,
    pub issuer_key: IrIssuerKey,
    pub source_id: String,
    pub market: IrMarket,
    pub family: IrDocumentFamily,
    pub title: String,
    pub language: Option<String>,
    pub filed_at: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrArtifactKey {
    pub document: IrDocumentKey,
    pub artifact_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrArtifactSource {
    ByteSource,
    LinkSource,
    EmbeddedSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrArtifactDescriptor {
    pub key: IrArtifactKey,
    pub source_id: String,
    pub kind: IrArtifactKind,
    pub content_type: Option<String>,
    pub source: IrArtifactSource,
    pub checksum_sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrFetchSupport {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrNormalizedFormat {
    Html,
    Pdf,
    Xbrl,
    Ixbrl,
    Xml,
    Txt,
    Csv,
    Json,
    Rss,
    Zip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrNormalizationSchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Default for IrNormalizationSchemaVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrNormalizationSupport {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IrNormalizationProvenance {
    pub source_type: Option<String>,
    pub source_ref: Option<String>,
    pub context_ref: Option<String>,
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrNormalizedSection {
    pub heading_level: Option<u8>,
    pub title: String,
    pub ordinal: usize,
    pub text_range: (usize, usize),
    pub provenance: IrNormalizationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrNormalizedTable {
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub provenance: IrNormalizationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrNormalizedAttachment {
    pub path: String,
    pub media_type: Option<String>,
    pub size_bytes: u64,
    pub checksum_sha256: Option<String>,
    pub provenance: IrNormalizationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrNormalizedContent {
    pub document_key: IrDocumentKey,
    pub artifact_key: IrArtifactKey,
    pub normalization_schema_version: IrNormalizationSchemaVersion,
    pub normalized_format: IrNormalizedFormat,
    pub normalized_text: String,
    pub sections: Vec<IrNormalizedSection>,
    pub tables: Vec<IrNormalizedTable>,
    pub extracted_attachments: Vec<IrNormalizedAttachment>,
    pub language_hints: Vec<String>,
    pub charset: Option<String>,
    pub provenance: IrNormalizationProvenance,
    pub support_level: IrNormalizationSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrAccessDecision {
    Allowed,
    ReviewRequired,
    Blocked,
}

pub fn normalize_alias(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

pub fn validate_document_artifact_pair(
    _family: IrDocumentFamily,
    _artifact: IrArtifactKind,
) -> bool {
    true
}
