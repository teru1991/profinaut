use ucel_core::IrAccessPolicyClass;
use ucel_ir::issuer_sites::{ensure_budget, ensure_policy_allowed, IssuerSitePolitenessPolicy};

#[test]
fn ir_issuer_sites_access_guard() {
    ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthAllowed).expect("allowed class should pass");
    let review = ensure_policy_allowed(IrAccessPolicyClass::FreePublicNoAuthReviewRequired).expect_err("review class should block by default");
    assert!(review.to_string().contains("review_required"));
    let blocked = ensure_policy_allowed(IrAccessPolicyClass::ExcludedPolicyBlocked).expect_err("blocked should fail-fast");
    assert!(blocked.to_string().contains("PolicyBlocked"));

    let budget_err = ensure_budget(4, 1, IssuerSitePolitenessPolicy::default()).expect_err("depth cap should be enforced");
    assert!(budget_err.to_string().contains("CrawlDepthExceeded"));
}
