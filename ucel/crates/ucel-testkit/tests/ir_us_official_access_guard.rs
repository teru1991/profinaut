use ucel_core::IrAccessPolicyClass;
use ucel_ir::us_official::access::{ensure_policy_allowed, UsPolitenessPolicy};

#[test]
fn ir_us_official_access_guard() {
    ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthAllowed).expect("allowed");
    let review = ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthReviewRequired)
        .expect_err("review required should not pass by default");
    assert!(review.to_string().contains("review_required"));

    let blocked = ensure_policy_allowed(IrAccessPolicyClass::ExcludedPolicyBlocked)
        .expect_err("blocked should fail");
    assert!(blocked.to_string().contains("PolicyBlocked") || blocked.to_string().contains("blocked"));

    let policy = UsPolitenessPolicy::default();
    assert!(policy.retry_budget > 0);
    assert!(policy.max_attachment_bytes > 0);
    assert!(!policy.user_agent.is_empty());
}
