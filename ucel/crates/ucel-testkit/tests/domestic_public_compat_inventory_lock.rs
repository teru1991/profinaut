use ucel_testkit::domestic_public_compat::{assert_domestic_public_final_compat, default_repo_root, load_inventory_and_lock, summarize_inventory};

#[test]
fn domestic_public_compat_inventory_lock_matches_inventory() {
    let root = default_repo_root();
    let (inv, lock) = load_inventory_and_lock(&root).expect("load inventory+lock");
    let (summary, _, stable) = summarize_inventory(&inv);
    assert_eq!(lock.stable_identifiers, stable, "stable IDs drift");
    assert_eq!(lock.counts.total_entries, summary.total_entries);
    assert_eq!(lock.counts.rest_entries, summary.rest_entries);
    assert_eq!(lock.counts.ws_entries, summary.ws_entries);
}

#[test]
fn domestic_public_compat_inventory_lock_requires_explicit_update() {
    let root = default_repo_root();
    assert_domestic_public_final_compat(&root).expect("lock and inventory must be in sync");
}
