use std::fs;
use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [base.join("../.."), base.join("../../..")];
    for c in candidates {
        if c.join("fixtures/support_bundle/manifest.json").exists() {
            return c;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn load_bundle_fixture(name: &str) -> serde_json::Value {
    let root = repo_root();
    let path = root.join("fixtures").join("support_bundle").join(name);
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn assert_hashes_are_real(bundle: &serde_json::Value) {
    let m = bundle.get("manifest").expect("manifest");
    for key in [
        "coverage_hash",
        "coverage_v2_hash",
        "ws_rules_hash",
        "catalog_hash",
        "policy_hash",
        "symbol_meta_hash",
        "execution_surface_hash",
        "runtime_capability_hash",
    ] {
        let v = m.get(key).and_then(|v| v.as_str()).unwrap_or_default();
        assert!(!v.is_empty(), "hash empty: {key}");
        assert_ne!(v, "unknown", "hash unknown: {key}");
    }
}
