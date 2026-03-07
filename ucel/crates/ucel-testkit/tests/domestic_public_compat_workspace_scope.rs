use std::collections::BTreeSet;
use std::fs;
use ucel_testkit::domestic_public_compat::{collect_workspace_domestic_venues, default_repo_root, load_inventory_and_lock};

#[test]
fn domestic_public_compat_workspace_scope_matches_inventory_and_docs() {
    let root = default_repo_root();
    let workspace = collect_workspace_domestic_venues(&root).expect("workspace venues");
    let (_inv, lock) = load_inventory_and_lock(&root).expect("load");
    let lock_venues: BTreeSet<_> = lock.venues.iter().cloned().collect();
    assert_eq!(workspace, lock_venues, "workspace/inventory drift");

    let policy = fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_change_management.md")).expect("policy");
    for v in &lock.venues {
        assert!(policy.contains(v), "venue missing in change policy: {v}");
    }
}
