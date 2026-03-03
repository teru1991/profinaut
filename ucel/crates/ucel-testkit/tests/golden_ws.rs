use ucel_testkit::fixtures::{discover_ws_cases, repo_root_from_manifest_dir};
use ucel_testkit::golden::run_ws_venue;

fn strict_venues(repo_root: &std::path::Path) -> Vec<String> {
    let mut venues = ucel_testkit::coverage_v2::load_strict_venues(repo_root)
        .expect("load strict_venues.json")
        .strict_ws_golden;
    venues.sort();
    venues
}

#[test]
fn golden_ws_all_strict_venues_are_verified() {
    let repo_root = repo_root_from_manifest_dir();
    let venues = strict_venues(&repo_root);
    assert!(!venues.is_empty(), "no strict venues discovered");

    for venue in venues {
        let cases = discover_ws_cases(&repo_root, &venue)
            .unwrap_or_else(|e| panic!("discover_ws_cases failed for venue={venue}: {e}"));
        if cases.is_empty() {
            continue;
        }
        let count = run_ws_venue(&repo_root, &venue)
            .unwrap_or_else(|e| panic!("golden ws failed for venue={venue}: {e}"));
        assert!(count > 0, "strict venue {venue} has no golden ws cases");
    }
}
