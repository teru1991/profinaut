use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn golden_ws_root() -> PathBuf {
    repo_root().join("ucel/fixtures/golden/ws")
}

fn strict_venues_v2() -> Vec<String> {
    let strict =
        ucel_testkit::coverage_v2::load_strict_venues(&repo_root()).expect("strict venues");
    let mut venues = strict.strict_ws_golden;
    venues.sort();
    venues
}

fn has_any_file_in_dir(dir: &std::path::Path) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file() {
            return true;
        }
        if p.is_dir() && has_any_file_in_dir(&p) {
            return true;
        }
    }
    false
}

#[test]
fn strict_venues_must_have_golden_fixtures() {
    let venues = strict_venues_v2();
    assert!(
        !venues.is_empty(),
        "no strict venues found in coverage_v2/strict_venues.json"
    );

    let mut missing: Vec<String> = Vec::new();
    for venue in &venues {
        let dir = golden_ws_root().join(venue);
        if !dir.is_dir() || !has_any_file_in_dir(&dir) {
            missing.push(venue.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "strict venues missing golden fixtures: {}",
        missing.join(", ")
    );
}

#[test]
fn strict_venues_v2_list_is_non_empty() {
    let venues = strict_venues_v2();
    assert!(!venues.is_empty(), "no strict venues found");
}
