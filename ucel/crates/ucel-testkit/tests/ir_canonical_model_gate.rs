use ucel_testkit::ir_canonical::{assert_inventory_is_canonical, parse_access_policy, repo_root};

#[test]
fn ir_canonical_model_gate() {
    let root = repo_root();
    assert_inventory_is_canonical(&root).expect("inventory must map to canonical model enums");
    assert!(parse_access_policy("free_public_noauth_allowed").is_some());
}
