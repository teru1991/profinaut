use ucel_testkit::ir_inventory::{assert_ir_access_policy_complete, load_ir_inventory, repo_root};

#[test]
fn ir_access_policy_gate() {
    let root = repo_root();
    let inv = load_ir_inventory(&root).expect("load inventory");
    assert_ir_access_policy_complete(&inv).expect("access policy must be complete");

    for s in inv.sources {
        assert!(!s.access_patterns.is_empty(), "source access patterns missing");
    }
}
