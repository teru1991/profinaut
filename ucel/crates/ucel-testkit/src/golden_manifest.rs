use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub generated_at: String,
    pub files: Vec<ManifestFile>,
}

pub fn golden_root(ucel_root: &Path) -> PathBuf {
    ucel_root.join("fixtures").join("golden")
}

pub fn load_golden_manifest(ucel_root: &Path) -> Manifest {
    let path = golden_root(ucel_root).join("manifest.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn read_file_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

pub fn deny_patterns() -> &'static [&'static str] {
    &[
        "AKIA",
        "BEGIN PRIVATE KEY",
        "PRIVATE KEY-----",
        "Authorization: Bearer ",
        "X-Api-Key",
        "x-api-key",
        "API_KEY",
        "api_key",
        "api_secret",
    ]
}

pub fn assert_no_denied_patterns(path: &str, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    for pat in deny_patterns() {
        assert!(!text.contains(pat), "denied pattern found in {path}: {pat}");
    }
}
