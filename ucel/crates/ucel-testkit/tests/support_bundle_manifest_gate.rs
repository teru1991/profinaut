use std::path::PathBuf;

fn ucel_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("ucel root")
}

#[test]
fn support_bundle_manifest_fixture_is_sane() {
    let root = ucel_root();
    let manifest = ucel_testkit::support_bundle_manifest::load_support_bundle_manifest(&root);
    assert_eq!(manifest.version, 1);
    assert!(!manifest.required_paths.is_empty());
    assert!(!manifest.deny_patterns.is_empty());

    for path in manifest.required_paths {
        assert!(!path.trim().is_empty(), "required path must not be empty");
    }
}
