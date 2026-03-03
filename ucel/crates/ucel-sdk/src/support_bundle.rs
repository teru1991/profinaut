use crate::config::SdkConfig;
use ucel_registry::hub::Hub;

pub fn generate_support_bundle(
    cfg: &SdkConfig,
    hub: &Hub,
    transport_diag: serde_json::Value,
) -> serde_json::Value {
    let hub_diag = ucel_registry::support_bundle::hub_bundle(hub);
    serde_json::json!({
        "metadata": {
            "version": "support_bundle_spec_v1",
            "run_id": cfg.run_id,
            "timestamp_unix": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default(),
        },
        "ssot": hub_diag.get("ssot").cloned().unwrap_or_else(|| serde_json::json!({})),
        "transport": transport_diag,
        "hub": hub_diag,
        "wal": {"recent_segments": [], "latency_stats": {}},
        "errors": {"recent": []}
    })
}
