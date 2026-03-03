use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupportBundleManifest {
    pub version: u32,
    pub required_paths: Vec<String>,
    pub deny_patterns: Vec<String>,
}

pub fn load_support_bundle_manifest(ucel_root: &Path) -> SupportBundleManifest {
    let path = ucel_root
        .join("fixtures")
        .join("support_bundle")
        .join("manifest.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn assert_no_denied_patterns(path: &str, bytes: &[u8], deny_patterns: &[String]) {
    let text = String::from_utf8_lossy(bytes);
    for pat in deny_patterns {
        assert!(
            !text.contains(pat),
            "denied pattern found in {}: {}",
            path,
            pat
        );
    }
}
