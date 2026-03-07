use serde::{Deserialize, Serialize};
use ucel_core::{IrIssuerKey, IrMarket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSiteSeed {
    pub source_id: String,
    pub issuer_key: IrIssuerKey,
    pub seed_url: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuerSiteSectionKind {
    IrTop,
    NewsArchive,
    FilingArchive,
    PresentationLibrary,
    FinancialResults,
    SustainabilityLibrary,
    GovernanceLibrary,
    MiscLibrary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSiteSelectorRule {
    pub section: IssuerSiteSectionKind,
    pub css: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSiteFeedDescriptor {
    pub endpoint: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSiteAttachmentRule {
    pub extensions: Vec<String>,
    pub allowed_content_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSiteProfile {
    pub source_id: String,
    pub market: IrMarket,
    pub issuer_key: IrIssuerKey,
    pub root_url: String,
    pub ir_index_candidates: Vec<String>,
    pub feeds: Vec<IssuerSiteFeedDescriptor>,
    pub selectors: Vec<IssuerSiteSelectorRule>,
    pub attachment_rule: IssuerSiteAttachmentRule,
    pub language_hints: Vec<String>,
    pub max_depth: usize,
    pub page_budget: usize,
}
