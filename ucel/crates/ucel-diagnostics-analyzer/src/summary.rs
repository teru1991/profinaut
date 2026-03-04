use crate::read::{BundleReadError, BundleReader, ParsedManifest};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum SummaryBuildError {
    #[error("bundle read error: {0}")]
    Read(#[from] BundleReadError),
    #[error("unsupported diag_semver: {0}")]
    Unsupported(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyzerSummary {
    pub bundle_id: String,
    pub created_at_rfc3339: String,
    pub diag_semver: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub top_paths: Vec<String>,
    pub extracted_signals: BTreeMap<String, String>,
}

pub fn analyze_tar_zst_bundle(
    bytes: Vec<u8>,
) -> Result<(ParsedManifest, AnalyzerSummary), SummaryBuildError> {
    let reader = BundleReader::from_bytes(bytes);
    let (manifest, files) = reader.read()?;

    let major = Version::parse(&manifest.diag_semver)
        .map(|v| v.major)
        .unwrap_or_default();
    if major != crate::SUPPORTED_DIAG_SEMVER_MAJOR {
        return Err(SummaryBuildError::Unsupported(manifest.diag_semver.clone()));
    }

    BundleReader::verify_integrity(&manifest, &files)?;

    let total_bytes = files.values().map(|v| v.len() as u64).sum();

    let mut top_paths: Vec<String> = files.keys().cloned().collect();
    top_paths.sort();
    top_paths.truncate(20);

    let mut extracted_signals = BTreeMap::new();

    if let Some(raw) = files.get("meta/diag_semver.txt") {
        extracted_signals.insert(
            "diag_semver_txt".to_string(),
            String::from_utf8_lossy(raw).trim().to_string(),
        );
    }

    if let Some(raw) = files.get("meta/info.json") {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(raw) {
            if let Some(k) = value.get("k").and_then(|v| v.as_str()) {
                extracted_signals.insert("meta_info_k".to_string(), k.to_string());
            }
        }
    }

    Ok((
        manifest.clone(),
        AnalyzerSummary {
            bundle_id: manifest.bundle_id.clone(),
            created_at_rfc3339: manifest.created_at_rfc3339.clone(),
            diag_semver: manifest.diag_semver.clone(),
            file_count: files.len(),
            total_bytes,
            top_paths,
            extracted_signals,
        },
    ))
}
