/// PR gate: every venue with `strict: true` in ucel/coverage/ (v1 schema)
/// must have a golden fixture directory containing at least one file.
///
/// Scope: ucel/coverage/ (Market Data H, v1 schema).
/// ucel/coverage_v2/ (v2 schema) has a separate golden policy tracked in UCEL-H-STRICT-001.
///
/// This test is intentionally minimal: it checks *existence*, not normalization correctness.
/// Normalization correctness for venues with subdirectory case fixtures is verified by golden_ws.rs.
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = ucel/crates/ucel-testkit
    // go up 3 levels: ucel-testkit -> crates -> ucel -> repo root
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

fn ucel_root() -> PathBuf {
    repo_root().join("ucel")
}

fn golden_ws_root() -> PathBuf {
    ucel_root().join("fixtures").join("golden").join("ws")
}

/// Read strict=true venue names from ucel/coverage/ YAML files (v1 schema).
fn strict_venues_v1() -> Vec<String> {
    let coverage_dir = ucel_root().join("coverage");

    let mut venues = Vec::new();
    let rd = std::fs::read_dir(&coverage_dir)
        .unwrap_or_else(|e| panic!("cannot read ucel/coverage/: {e}"));

    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let txt = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => panic!("read {}: {e}", path.display()),
        };
        let v: serde_yaml::Value = match serde_yaml::from_str(&txt) {
            Ok(v) => v,
            Err(e) => panic!("parse {}: {e}", path.display()),
        };
        let strict = v.get("strict").and_then(|x| x.as_bool()).unwrap_or(false);
        if !strict {
            continue;
        }
        let venue = v
            .get("venue")
            .and_then(|x| x.as_str())
            .unwrap_or_else(|| panic!("missing venue field in {}", path.display()))
            .to_string();
        venues.push(venue);
    }

    venues.sort();
    venues
}

/// Return true if the venue has at least one file in its golden/ws/<venue>/ directory.
fn venue_has_golden(venue: &str) -> bool {
    let dir = golden_ws_root().join(venue);
    if !dir.is_dir() {
        return false;
    }
    has_any_file_in_dir(&dir)
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

/// Gate: strict venues must have golden fixture directories.
///
/// If this test fails: add at minimum a stub file to ucel/fixtures/golden/ws/<venue>/
/// (e.g. stub.json) and eventually a full golden case subdirectory.
#[test]
fn strict_venues_must_have_golden_fixtures() {
    let venues = strict_venues_v1();
    assert!(
        !venues.is_empty(),
        "no strict=true venues found in ucel/coverage/ — strict policy may be broken"
    );

    let mut missing: Vec<String> = Vec::new();
    for venue in &venues {
        if !venue_has_golden(venue) {
            missing.push(venue.clone());
        }
    }

    if !missing.is_empty() {
        panic!(
            "strict venues missing golden fixtures in ucel/fixtures/golden/ws/<venue>/:\n  {}\n\
             Add at least stub.json to each directory to satisfy this gate.",
            missing.join(", ")
        );
    }
}

/// Sanity: all found strict venues are listed (for easy CI output).
#[test]
fn strict_venues_v1_list_is_non_empty() {
    let venues = strict_venues_v1();
    assert!(
        !venues.is_empty(),
        "no strict venues found — ucel/coverage/ may be missing or all files are strict: false"
    );
    // Print for visibility in CI
    println!("strict venues ({}):", venues.len());
    for v in &venues {
        println!("  - {v}");
    }
}
