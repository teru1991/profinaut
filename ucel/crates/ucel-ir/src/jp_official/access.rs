use super::errors::{JpOfficialError, JpOfficialErrorCode};
use ucel_core::IrAccessPolicyClass;

#[derive(Debug, Clone, Copy)]
pub struct JpPolitenessPolicy {
    pub concurrency_cap: usize,
    pub retry_budget: u8,
    pub base_backoff_ms: u64,
    pub max_attachment_bytes: u64,
}

impl Default for JpPolitenessPolicy {
    fn default() -> Self {
        Self {
            concurrency_cap: 2,
            retry_budget: 3,
            base_backoff_ms: 250,
            max_attachment_bytes: 5 * 1024 * 1024,
        }
    }
}

pub fn ensure_policy_allowed(policy: IrAccessPolicyClass) -> Result<(), JpOfficialError> {
    match policy {
        IrAccessPolicyClass::FreePublicNoAuthAllowed => Ok(()),
        IrAccessPolicyClass::FreePublicNoAuthReviewRequired => Err(JpOfficialError::new(
            JpOfficialErrorCode::ReviewRequired,
            "review_required source needs explicit approval",
        )),
        IrAccessPolicyClass::ExcludedPaidOrContract
        | IrAccessPolicyClass::ExcludedLoginRequired
        | IrAccessPolicyClass::ExcludedPolicyBlocked => Err(JpOfficialError::new(
            JpOfficialErrorCode::PolicyBlocked,
            "source is blocked by policy",
        )),
    }
}

pub fn ensure_attachment_size(
    size: u64,
    policy: JpPolitenessPolicy,
) -> Result<(), JpOfficialError> {
    if size > policy.max_attachment_bytes {
        return Err(JpOfficialError::new(
            JpOfficialErrorCode::OversizedArtifact,
            format!("artifact too large: {size}"),
        ));
    }
    Ok(())
}
