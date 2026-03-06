use crate::config::SdkConfig;
use sha2::{Digest, Sha256};
use ucel_registry::hub::Hub;

pub fn generate_support_bundle(
    cfg: &SdkConfig,
    hub: &Hub,
    transport_diag: serde_json::Value,
) -> serde_json::Value {
    let hub_diag = ucel_registry::support_bundle::hub_bundle(hub);
    let ssot = hub_diag.get("ssot").cloned().unwrap_or_else(|| serde_json::json!({}));

    let runtime_digest = hash_value(&serde_json::json!({
        "transport": transport_diag,
        "hub": hub_diag
    }));

    let manifest = serde_json::json!({
        "diag_semver": ucel_diagnostics_core::DIAG_SEMVER_STR,
        "generated_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default(),
        "generator_id": "ucel-sdk",
        "build_info": env!("CARGO_PKG_VERSION"),
        "coverage_hash": ssot.get("coverage_hash").cloned().unwrap_or_default(),
        "coverage_v2_hash": ssot.get("coverage_v2_hash").cloned().unwrap_or_default(),
        "ws_rules_hash": ssot.get("ws_rules_hash").cloned().unwrap_or_default(),
        "catalog_hash": ssot.get("catalog_hash").cloned().unwrap_or_default(),
        "policy_hash": ssot.get("policy_hash").cloned().unwrap_or_default(),
        "symbol_meta_hash": ssot.get("symbol_meta_hash").cloned().unwrap_or_default(),
        "execution_surface_hash": ssot.get("execution_surface_hash").cloned().unwrap_or_default(),
        "runtime_capability_hash": ssot.get("runtime_capability_hash").cloned().unwrap_or_default(),
        "bundle_redaction_version": "v1",
        "runtime_capabilities": runtime_digest,
    });

    serde_json::json!({
        "metadata": {
            "version": "support_bundle_spec_v1",
            "run_id": cfg.run_id,
            "timestamp_unix": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default(),
        },
        "manifest": manifest,
        "ssot": ssot,
        "transport": transport_diag,
        "hub": hub_diag,
        "wal": {"recent_segments": [], "latency_stats": {}},
        "errors": {"recent": []}
    })
}

fn hash_value(v: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    let stable = serde_json::to_string(v).unwrap_or_default();
    hasher.update(stable.as_bytes());
    hex::encode(hasher.finalize())
}
