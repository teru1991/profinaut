use super::{InvokerError, OperationId, VenueId};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoverageSupport {
    #[default]
    Supported,
    NotSupported,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageManifest {
    pub venue: String,
    pub strict: bool,
    pub entries: Vec<CoverageEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageEntry {
    pub id: String,
    pub implemented: bool,
    pub tested: bool,
    #[serde(default)]
    pub support: CoverageSupport,
    #[serde(default)]
    pub strict: Option<bool>,
}

impl CoverageEntry {
    pub fn is_supported(&self) -> bool {
        self.support == CoverageSupport::Supported
    }

    pub fn effective_strict(&self, manifest_strict: bool) -> bool {
        self.strict.unwrap_or(manifest_strict)
    }
}

pub fn discover(root: &Path) -> Result<Vec<(VenueId, CoverageManifest)>, InvokerError> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let p = entry?.path();
        if p.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(&p)?;
        let manifest: CoverageManifest = serde_yaml::from_str(&raw)?;
        let venue: VenueId = manifest.venue.parse()?;
        out.push((venue, manifest));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

pub fn ids(manifest: &CoverageManifest) -> Result<Vec<OperationId>, InvokerError> {
    manifest.entries.iter().map(|e| e.id.parse()).collect()
}

pub fn ids_supported(manifest: &CoverageManifest) -> Result<Vec<OperationId>, InvokerError> {
    manifest
        .entries
        .iter()
        .filter(|e| e.is_supported())
        .map(|e| e.id.parse())
        .collect()
}
