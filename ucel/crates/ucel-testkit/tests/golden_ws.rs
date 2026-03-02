use std::fs;

use ucel_testkit::fixtures::repo_root_from_manifest_dir;
use ucel_testkit::golden::run_ws_venue;

fn strict_venues(repo_root: &std::path::Path) -> Vec<String> {
    let coverage_dir = repo_root.join("ucel").join("coverage");
    let mut venues = Vec::new();

    for entry in fs::read_dir(&coverage_dir).expect("read coverage dir") {
        let entry = entry.expect("read coverage entry");
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }

        let raw = fs::read_to_string(&path).expect("read coverage yaml");
        let value: serde_yaml::Value = serde_yaml::from_str(&raw).expect("parse coverage yaml");
        let strict = value
            .get("strict")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if strict {
            let venue = value
                .get("venue")
                .and_then(|v| v.as_str())
                .expect("coverage venue")
                .to_string();
            venues.push(venue);
        }
    }

    venues.sort();
    venues
}

#[test]
fn golden_ws_all_strict_venues_are_verified() {
    let repo_root = repo_root_from_manifest_dir();
    let venues = strict_venues(&repo_root);
    assert!(
        !venues.is_empty(),
        "no strict venues discovered from coverage"
    );

    for venue in venues {
        let count = run_ws_venue(&repo_root, &venue)
            .unwrap_or_else(|e| panic!("golden ws failed for venue={venue}: {e}"));
        assert!(count > 0, "strict venue {venue} has no golden ws cases");
    }
}
