use ucel_core::{validate_manifest_hash_presence, BundleManifest};

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid manifest: {0}")]
    Invalid(String),
}

pub fn parse_manifest_bytes(bytes: &[u8]) -> Result<BundleManifest, ManifestError> {
    let manifest: BundleManifest = serde_json::from_slice(bytes)?;
    validate_manifest_hash_presence(&manifest).map_err(ManifestError::Invalid)?;
    Ok(manifest)
}

pub fn manifest_to_pretty_json(manifest: &BundleManifest) -> Result<Vec<u8>, ManifestError> {
    Ok(serde_json::to_vec_pretty(manifest)?)
}
