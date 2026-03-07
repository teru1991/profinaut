use ucel_testkit::domestic_public_compat::{collect_fixture_coverage, default_repo_root, load_inventory_and_lock};

#[test]
fn domestic_public_compat_required_goldens_exist() {
    let root = default_repo_root();
    let got = collect_fixture_coverage(&root).expect("collect fixture coverage");
    assert_eq!(got.len(), 5, "missing golden fixture family: {got:?}");
    let (_inv, lock) = load_inventory_and_lock(&root).expect("load");
    assert!(!lock.stable_identifiers.is_empty(), "lock must remain immutable input for goldens");
}
