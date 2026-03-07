use super::errors::{UsOfficialError, UsOfficialErrorCode};
use ucel_core::IrAccessPolicyClass;

#[derive(Debug, Clone, Copy)]
pub struct UsPolitenessPolicy {
    pub user_agent: &'static str,
    pub concurrency_cap: usize,
    pub retry_budget: u8,
    pub base_backoff_ms: u64,
    pub max_attachment_bytes: u64,
}

impl Default for UsPolitenessPolicy {
    fn default() -> Self {
        Self {
            user_agent: "UCEL-IR/014D (+https://example.invalid/ucel)",
            concurrency_cap: 2,
            retry_budget: 3,
            base_backoff_ms: 300,
            max_attachment_bytes: 8 * 1024 * 1024,
        }
    }
}

pub fn ensure_policy_allowed(policy: IrAccessPolicyClass) -> Result<(), UsOfficialError> {
    match policy {
        IrAccessPolicyClass::FreePublicNoAuthAllowed => Ok(()),
        IrAccessPolicyClass::FreePublicNoAuthReviewRequired => Err(UsOfficialError::new(
            UsOfficialErrorCode::ReviewRequired,
            "review_required source needs explicit approval",
        )),
        IrAccessPolicyClass::ExcludedPaidOrContract
        | IrAccessPolicyClass::ExcludedLoginRequired
        | IrAccessPolicyClass::ExcludedPolicyBlocked => Err(UsOfficialError::new(
            UsOfficialErrorCode::PolicyBlocked,
            "source is blocked by policy",
        )),
    }
}

pub fn ensure_attachment_size(
    size: u64,
    policy: UsPolitenessPolicy,
) -> Result<(), UsOfficialError> {
    if size > policy.max_attachment_bytes {
        return Err(UsOfficialError::new(
            UsOfficialErrorCode::OversizedArtifact,
            format!("artifact too large: {size}"),
        ));
    }
    Ok(())
}
