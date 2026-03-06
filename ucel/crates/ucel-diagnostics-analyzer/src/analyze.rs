use crate::compat::evaluate_compatibility;
use crate::drift::detect_runbook_drift;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("missing manifest")]
    MissingManifest,
    #[error("manifest parse: {0}")]
    Manifest(String),
    #[error("redaction violation: {0}")]
    Redaction(String),
}

pub fn analyze_support_bundle_value(bundle: &serde_json::Value, repo_root: &Path) -> Result<(serde_json::Value, serde_json::Value, serde_json::Value), AnalyzeError> {
    let manifest = bundle.get("manifest").ok_or(AnalyzeError::MissingManifest)?;

    let diag_semver = manifest
        .get("diag_semver")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AnalyzeError::Manifest("diag_semver missing".into()))?;

    let text = serde_json::to_string(bundle).unwrap_or_default();
    if let Some(pat) = ucel_diagnostics_core::redaction::contains_denied_pattern(&text) {
        return Err(AnalyzeError::Redaction(pat.into()));
    }

    let comp = evaluate_compatibility(diag_semver);
    let findings = detect_runbook_drift(repo_root);

    let summary = serde_json::json!({
        "diag_semver": diag_semver,
        "generator_id": manifest.get("generator_id").cloned().unwrap_or_default(),
        "build_info": manifest.get("build_info").cloned().unwrap_or_default(),
        "runtime_capability_hash": manifest.get("runtime_capability_hash").cloned().unwrap_or_default(),
        "coverage_hash": manifest.get("coverage_hash").cloned().unwrap_or_default(),
        "coverage_v2_hash": manifest.get("coverage_v2_hash").cloned().unwrap_or_default(),
        "ws_rules_hash": manifest.get("ws_rules_hash").cloned().unwrap_or_default(),
        "catalog_hash": manifest.get("catalog_hash").cloned().unwrap_or_default(),
        "policy_hash": manifest.get("policy_hash").cloned().unwrap_or_default(),
        "symbol_meta_hash": manifest.get("symbol_meta_hash").cloned().unwrap_or_default(),
        "execution_surface_hash": manifest.get("execution_surface_hash").cloned().unwrap_or_default(),
    });

    let compat = serde_json::json!({
        "status": format!("{:?}", comp),
        "supported_major": crate::SUPPORTED_DIAG_SEMVER_MAJOR,
        "bundle_diag_semver": diag_semver,
    });

    let drift = serde_json::json!({
        "findings": findings,
    });

    Ok((summary, compat, drift))
}
