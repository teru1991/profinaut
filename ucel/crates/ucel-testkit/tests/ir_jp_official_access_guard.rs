use ucel_core::IrAccessPolicyClass;
use ucel_ir::jp_official::{ensure_policy_allowed, JpPolitenessPolicy};

#[test]
fn ir_jp_official_access_guard() {
    assert!(ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthAllowed).is_ok());
    assert!(ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthReviewRequired).is_err());
    assert!(ensure_policy_allowed(IrAccessPolicyClass::ExcludedPolicyBlocked).is_err());

    let policy = JpPolitenessPolicy::default();
    assert!(policy.retry_budget > 0);
    assert!(policy.base_backoff_ms > 0);
    assert!(policy.max_attachment_bytes > 0);
}
