use ucel_testkit::domestic_public_inventory::{
    assert_domestic_public_inventory_complete, compare_inventory_to_repo,
    load_domestic_public_inventory, repo_root,
};

#[test]
fn domestic_public_inventory_gate() {
    let root = repo_root();
    let inv = assert_domestic_public_inventory_complete(&root).expect("inventory gate should pass");
    assert!(!inv.entries.is_empty(), "inventory must not be empty");
}

#[test]
fn domestic_public_inventory_has_no_public_private_mix() {
    let root = repo_root();
    let inv = load_domestic_public_inventory(&root).expect("load inventory");
    let diff = compare_inventory_to_repo(&root, &inv);
    assert!(
        diff.non_public_auth.is_empty(),
        "non_public_auth={:?}",
        diff.non_public_auth
    );
    assert!(
        diff.duplicate_keys.is_empty(),
        "duplicate_keys={:?}",
        diff.duplicate_keys
    );
}
