use std::collections::BTreeMap;
use ucel_core::{AnalyzerSummary, BundleManifest, CompatibilityStatus, DriftFinding};

pub fn manifest_summary(_manifest: &BundleManifest, compatibility: CompatibilityStatus, findings: Vec<DriftFinding>) -> AnalyzerSummary {
    AnalyzerSummary { compatibility, findings }
}

pub fn summary_json(manifest: &BundleManifest, compatibility: CompatibilityStatus, findings: Vec<DriftFinding>) -> serde_json::Value {
    let mut hashes = BTreeMap::new();
    hashes.insert("coverage_hash", manifest.hashes.coverage_hash.clone());
    hashes.insert("coverage_v2_hash", manifest.hashes.coverage_v2_hash.clone());
    hashes.insert("ws_rules_hash", manifest.hashes.ws_rules_hash.clone());
    hashes.insert("catalog_hash", manifest.hashes.catalog_hash.clone());
    hashes.insert("policy_hash", manifest.hashes.policy_hash.clone());
    hashes.insert("symbol_meta_hash", manifest.hashes.symbol_meta_hash.clone());
    hashes.insert("execution_surface_hash", manifest.hashes.execution_surface_hash.clone());
    hashes.insert("runtime_capability_hash", manifest.hashes.runtime_capability_hash.clone());
    serde_json::json!({
        "diag_semver": manifest.diag_semver.0,
        "generated_at": manifest.generated_at,
        "generator_id": manifest.generator.generator_id,
        "build_info": manifest.generator.build_info,
        "runtime_digest": manifest.runtime.digest,
        "compatibility": format!("{:?}", compatibility),
        "hashes": hashes,
        "findings": findings,
    })
}
