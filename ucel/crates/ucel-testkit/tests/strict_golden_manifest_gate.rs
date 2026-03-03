use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn ucel_root() -> PathBuf {
    repo_root().join("ucel")
}

fn strict_venues_from_coverage_v2(repo_root: &std::path::Path) -> Vec<String> {
    let mut venues = ucel_testkit::coverage_v2::load_strict_venues(repo_root)
        .expect("load strict_venues.json")
        .strict_ws_golden;
    venues.sort();
    venues
}

fn required_ws_files(venue: &str) -> [String; 2] {
    [
        format!("ws/{venue}/raw.json"),
        format!("ws/{venue}/expected.normalized.json"),
    ]
}

#[test]
fn strict_venues_must_have_required_golden_files_in_manifest() {
    let ucel_root = ucel_root();
    let manifest = ucel_testkit::golden_manifest::load_golden_manifest(&ucel_root);
    assert_eq!(manifest.version, 1, "manifest version mismatch");

    let files: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|f| (f.path.clone(), (&f.sha256, f.bytes)))
        .collect();

    let venues = strict_venues_from_coverage_v2(&repo_root());
    assert!(
        !venues.is_empty(),
        "no strict venues found in strict_venues.json"
    );

    let mut missing = Vec::new();
    for venue in venues {
        for req in required_ws_files(&venue) {
            if !files.contains_key(&req) {
                missing.push(req);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing required strict venue files in manifest: {missing:?}"
    );
}

#[test]
fn golden_manifest_sha_size_and_redaction_checks() {
    let ucel_root = ucel_root();
    let manifest = ucel_testkit::golden_manifest::load_golden_manifest(&ucel_root);

    for f in manifest.files {
        let full_path = ucel_testkit::golden_manifest::golden_root(&ucel_root).join(&f.path);
        let bytes = ucel_testkit::golden_manifest::read_file_bytes(&full_path);
        ucel_testkit::golden_manifest::assert_no_denied_patterns(&f.path, &bytes);

        let sha = ucel_testkit::golden_manifest::sha256_bytes(&bytes);
        assert_eq!(sha, f.sha256, "sha mismatch for {}", f.path);
        assert_eq!(bytes.len() as u64, f.bytes, "bytes mismatch for {}", f.path);
        assert!(
            f.bytes <= 1024 * 1024,
            "fixture too large (>1MiB): {}",
            f.path
        );
    }
}
