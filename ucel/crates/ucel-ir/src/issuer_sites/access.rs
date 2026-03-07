use super::errors::{IssuerSiteError, IssuerSiteErrorCode};
use ucel_core::IrAccessPolicyClass;

#[derive(Debug, Clone, Copy)]
pub struct IssuerSitePolitenessPolicy {
    pub concurrency_cap: usize,
    pub retry_budget: u8,
    pub crawl_depth_cap: usize,
    pub page_budget: usize,
    pub base_backoff_ms: u64,
    pub max_attachment_bytes: u64,
}

impl Default for IssuerSitePolitenessPolicy {
    fn default() -> Self {
        Self {
            concurrency_cap: 2,
            retry_budget: 3,
            crawl_depth_cap: 3,
            page_budget: 16,
            base_backoff_ms: 250,
            max_attachment_bytes: 8 * 1024 * 1024,
        }
    }
}

pub fn ensure_policy_allowed(policy: IrAccessPolicyClass) -> Result<(), IssuerSiteError> {
    match policy {
        IrAccessPolicyClass::FreePublicNoAuthAllowed => Ok(()),
        IrAccessPolicyClass::FreePublicNoAuthReviewRequired => Err(IssuerSiteError::new(
            IssuerSiteErrorCode::ReviewRequired,
            "review_required source needs explicit approval",
        )),
        IrAccessPolicyClass::ExcludedPaidOrContract
        | IrAccessPolicyClass::ExcludedLoginRequired
        | IrAccessPolicyClass::ExcludedPolicyBlocked => Err(IssuerSiteError::new(
            IssuerSiteErrorCode::PolicyBlocked,
            "source is blocked by policy",
        )),
    }
}

pub fn ensure_budget(depth: usize, pages: usize, p: IssuerSitePolitenessPolicy) -> Result<(), IssuerSiteError> {
    if depth > p.crawl_depth_cap {
        return Err(IssuerSiteError::new(
            IssuerSiteErrorCode::CrawlDepthExceeded,
            format!("crawl depth exceeded: {depth}"),
        ));
    }
    if pages > p.page_budget {
        return Err(IssuerSiteError::new(
            IssuerSiteErrorCode::DiscoveryBudgetExceeded,
            format!("page budget exceeded: {pages}"),
        ));
    }
    Ok(())
}

pub fn ensure_attachment_size(size: u64, p: IssuerSitePolitenessPolicy) -> Result<(), IssuerSiteError> {
    if size > p.max_attachment_bytes {
        return Err(IssuerSiteError::new(
            IssuerSiteErrorCode::OversizedArtifact,
            format!("artifact too large: {size}"),
        ));
    }
    Ok(())
}
