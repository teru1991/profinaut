use std::collections::BTreeSet;
use ucel_testkit::ir_inventory::{assert_ir_inventory_complete, repo_root};

#[test]
fn ir_inventory_gate() {
    let root = repo_root();
    let inv = assert_ir_inventory_complete(&root).expect("ir inventory must be complete");

    let mut uniq = BTreeSet::new();
    for s in inv.sources {
        assert!(
            uniq.insert(format!("{}|{}", s.market, s.source_id)),
            "duplicate source"
        );
    }
}
