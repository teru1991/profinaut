use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleManifestFile {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundlePolicySummary {
    pub archive_format: String,
    pub max_total_bytes: u64,
    pub max_files: usize,
    pub max_single_file_bytes: u64,
    pub max_path_len: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    pub schema_version: u8,
    pub bundle_id: String,
    pub created_at: String,
    pub diag_semver: String,
    pub files: Vec<BundleManifestFile>,
    pub policy_summary: BundlePolicySummary,
    pub notes: Vec<String>,
}
