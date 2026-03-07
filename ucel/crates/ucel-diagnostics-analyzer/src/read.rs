use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum BundleReadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zstd decode: {0}")]
    Zstd(String),
    #[error("tar: {0}")]
    Tar(String),
    #[error("missing manifest.json in archive")]
    MissingManifest,
    #[error("manifest json invalid: {0}")]
    ManifestJson(#[from] serde_json::Error),
    #[error("manifest integrity mismatch for {path}")]
    IntegrityMismatch { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub size_bytes: u64,
    #[serde(alias = "sha256")]
    pub sha256_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedManifest {
    pub bundle_id: String,
    #[serde(alias = "created_at")]
    pub created_at_rfc3339: String,
    pub diag_semver: String,
    pub files: Vec<ManifestFile>,
}

pub struct BundleReader {
    bytes: Vec<u8>,
}

impl BundleReader {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn read(&self) -> Result<(ParsedManifest, BTreeMap<String, Vec<u8>>), BundleReadError> {
        let tar_bytes = decode_zstd(&self.bytes)?;
        let mut ar = tar::Archive::new(std::io::Cursor::new(tar_bytes));

        let mut manifest: Option<ParsedManifest> = None;
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();

        let entries = ar
            .entries()
            .map_err(|e| BundleReadError::Tar(e.to_string()))?;
        for entry in entries {
            let mut entry = entry.map_err(|e| BundleReadError::Tar(e.to_string()))?;
            let path = entry
                .path()
                .map_err(|e| BundleReadError::Tar(e.to_string()))?
                .to_string_lossy()
                .to_string();

            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;

            if path == "manifest.json" {
                let parsed: ParsedManifest = serde_json::from_slice(&buf)?;
                manifest = Some(parsed);
            } else {
                files.insert(path, buf);
            }
        }

        let manifest = manifest.ok_or(BundleReadError::MissingManifest)?;
        Ok((manifest, files))
    }

    pub fn verify_integrity(
        manifest: &ParsedManifest,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<(), BundleReadError> {
        for file in &manifest.files {
            let bytes =
                files
                    .get(&file.path)
                    .ok_or_else(|| BundleReadError::IntegrityMismatch {
                        path: file.path.clone(),
                    })?;

            if bytes.len() as u64 != file.size_bytes {
                return Err(BundleReadError::IntegrityMismatch {
                    path: file.path.clone(),
                });
            }

            if sha256_hex(bytes) != file.sha256_hex {
                return Err(BundleReadError::IntegrityMismatch {
                    path: file.path.clone(),
                });
            }
        }

        Ok(())
    }
}

fn decode_zstd(bytes: &[u8]) -> Result<Vec<u8>, BundleReadError> {
    let mut decoder = zstd::Decoder::new(std::io::Cursor::new(bytes))
        .map_err(|e| BundleReadError::Zstd(e.to_string()))?;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
