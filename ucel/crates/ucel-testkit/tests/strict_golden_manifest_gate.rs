use std::collections::BTreeMap;
use std::path::PathBuf;

fn ucel_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("ucel root")
}

fn strict_venues_from_coverage(root: &std::path::Path) -> Vec<String> {
    let mut venues = Vec::new();
    let coverage_dir = root.join("coverage");
    let rd = std::fs::read_dir(&coverage_dir).expect("read coverage");

    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).expect("read coverage yaml");
        let value: serde_yaml::Value = serde_yaml::from_str(&raw).expect("parse coverage yaml");
        if value.get("strict").and_then(|x| x.as_bool()) == Some(true) {
            let venue = value
                .get("venue")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| panic!("missing venue in {}", path.display()));
            venues.push(venue.to_string());
        }
    }

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
    let root = ucel_root();
    let manifest = ucel_testkit::golden_manifest::load_golden_manifest(&root);
    assert_eq!(manifest.version, 1, "manifest version mismatch");

    let files: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|f| (f.path.clone(), (&f.sha256, f.bytes)))
        .collect();

    let venues = strict_venues_from_coverage(&root);
    assert!(
        !venues.is_empty(),
        "no strict venues found in ucel/coverage"
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
    let root = ucel_root();
    let manifest = ucel_testkit::golden_manifest::load_golden_manifest(&root);

    for f in manifest.files {
        let full_path = ucel_testkit::golden_manifest::golden_root(&root).join(&f.path);
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
