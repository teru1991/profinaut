use ucel_testkit::domestic_public_compat::{collect_route_reachability, default_repo_root, load_inventory_and_lock};

#[test]
fn domestic_public_compat_all_inventory_entries_are_reachable() {
    let root = default_repo_root();
    let (inv, _) = load_inventory_and_lock(&root).expect("load inventory");
    let routes = collect_route_reachability().expect("route collect");

    for e in inv.entries {
        assert_eq!(e.current_repo_status, "implemented", "non-implemented entry: {}", e.public_id);
        let id = format!("{}|{}|{}", e.venue, e.api_kind, e.public_id);
        assert!(routes.contains(&id), "missing route for {id}");
    }
}
