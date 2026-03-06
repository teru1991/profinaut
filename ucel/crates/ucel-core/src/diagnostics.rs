use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsSemver(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleHashSet {
    pub coverage_hash: String,
    pub coverage_v2_hash: String,
    pub ws_rules_hash: String,
    pub catalog_hash: String,
    pub policy_hash: String,
    pub symbol_meta_hash: String,
    pub execution_surface_hash: String,
    pub runtime_capability_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleGeneratorInfo {
    pub generator_id: String,
    pub build_info: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilitiesDigest {
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleManifest {
    pub diag_semver: DiagnosticsSemver,
    pub generated_at: String,
    pub generator: BundleGeneratorInfo,
    pub hashes: BundleHashSet,
    pub runtime: RuntimeCapabilitiesDigest,
    pub bundle_redaction_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Supported,
    SupportedWithWarnings,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsSupport {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftFinding {
    pub level: String,
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzerSummary {
    pub compatibility: CompatibilityStatus,
    pub findings: Vec<DriftFinding>,
}

pub fn compare_semver(a: &DiagnosticsSemver, b: &DiagnosticsSemver) -> Option<std::cmp::Ordering> {
    let av = Version::parse(&a.0).ok()?;
    let bv = Version::parse(&b.0).ok()?;
    Some(av.cmp(&bv))
}

pub fn validate_manifest_hash_presence(manifest: &BundleManifest) -> Result<(), String> {
    let vals = [
        &manifest.hashes.coverage_hash,
        &manifest.hashes.coverage_v2_hash,
        &manifest.hashes.ws_rules_hash,
        &manifest.hashes.catalog_hash,
        &manifest.hashes.policy_hash,
        &manifest.hashes.symbol_meta_hash,
        &manifest.hashes.execution_surface_hash,
        &manifest.hashes.runtime_capability_hash,
        &manifest.runtime.digest,
    ];
    if vals.iter().any(|v| v.trim().is_empty() || *v == "unknown") {
        return Err("bundle manifest contains missing/unknown hash".into());
    }
    Ok(())
}
