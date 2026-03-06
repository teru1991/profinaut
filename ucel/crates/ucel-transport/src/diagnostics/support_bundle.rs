use crate::health::TransportHealth;
use crate::obs::{export_prometheus::encode_prometheus_text, StabilityEventRing, TransportMetrics};

#[derive(Debug, Clone)]
pub struct SupportBundleInput {
    pub exchange_id: String,
    pub conn_id: String,
    pub health: TransportHealth,
    pub metrics: std::sync::Arc<TransportMetrics>,
    pub events: std::sync::Arc<StabilityEventRing>,
    pub rules_snapshot: serde_json::Value,
}

fn support_bundle_manifest() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/support_bundle/manifest.json"
    ))
    .expect("parse support bundle fixture manifest")
}

pub fn build_support_bundle(input: SupportBundleInput) -> serde_json::Value {
    let events_tail = input.events.snapshot();
    let events_text = serde_json::to_string(&events_tail).unwrap_or_else(|_| "[]".to_string());
    let metrics_prom = encode_prometheus_text(&input.metrics);

    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let hashes = ucel_diagnostics_core::default_hash_set(&repo_root).unwrap_or_else(|_| {
        ucel_core::BundleHashSet {
            coverage_hash: "fallback".to_string(),
            coverage_v2_hash: "fallback".to_string(),
            ws_rules_hash: "fallback".to_string(),
            catalog_hash: "fallback".to_string(),
            policy_hash: "fallback".to_string(),
            symbol_meta_hash: "fallback".to_string(),
            execution_surface_hash: "fallback".to_string(),
            runtime_capability_hash: "fallback".to_string(),
        }
    });

    let mut manifest = support_bundle_manifest();
    if let Some(obj) = manifest.as_object_mut() {
        obj.insert(
            "diag_semver".to_string(),
            serde_json::Value::String(ucel_diagnostics_core::DIAG_SEMVER_STR.to_string()),
        );
        obj.insert(
            "coverage_hash".to_string(),
            serde_json::Value::String(hashes.coverage_hash.clone()),
        );
        obj.insert(
            "coverage_v2_hash".to_string(),
            serde_json::Value::String(hashes.coverage_v2_hash.clone()),
        );
        obj.insert(
            "ws_rules_hash".to_string(),
            serde_json::Value::String(hashes.ws_rules_hash.clone()),
        );
        obj.insert(
            "catalog_hash".to_string(),
            serde_json::Value::String(hashes.catalog_hash.clone()),
        );
        obj.insert(
            "policy_hash".to_string(),
            serde_json::Value::String(hashes.policy_hash.clone()),
        );
        obj.insert(
            "symbol_meta_hash".to_string(),
            serde_json::Value::String(hashes.symbol_meta_hash.clone()),
        );
        obj.insert(
            "execution_surface_hash".to_string(),
            serde_json::Value::String(hashes.execution_surface_hash.clone()),
        );
        obj.insert(
            "runtime_capability_hash".to_string(),
            serde_json::Value::String(hashes.runtime_capability_hash.clone()),
        );
    }

    serde_json::json!({
        "metadata": {
            "exchange_id": input.exchange_id,
            "conn_id": input.conn_id,
        },
        "ssot": {
            "rules_version": "v1",
            "coverage_hash": hashes.coverage_hash,
            "coverage_v2_hash": hashes.coverage_v2_hash,
            "ws_rules_hash": hashes.ws_rules_hash,
            "catalog_hash": hashes.catalog_hash,
            "policy_hash": hashes.policy_hash,
            "symbol_meta_hash": hashes.symbol_meta_hash,
            "execution_surface_hash": hashes.execution_surface_hash,
            "runtime_capability_hash": hashes.runtime_capability_hash,
        },
        "transport": {
            "health": input.health,
            "metrics": {
                "reconnect_attempts": input.metrics.reconnect_attempts.load(std::sync::atomic::Ordering::Relaxed),
                "reconnect_success": input.metrics.reconnect_success.load(std::sync::atomic::Ordering::Relaxed),
                "reconnect_failure": input.metrics.reconnect_failure.load(std::sync::atomic::Ordering::Relaxed),
                "breaker_open": input.metrics.breaker_open.load(std::sync::atomic::Ordering::Relaxed),
                "stale_requeued": input.metrics.stale_requeued.load(std::sync::atomic::Ordering::Relaxed),
                "outq_dropped": input.metrics.outq_dropped.load(std::sync::atomic::Ordering::Relaxed),
                "outq_spilled": input.metrics.outq_spilled.load(std::sync::atomic::Ordering::Relaxed),
                "rl_penalty_applied": input.metrics.rl_penalty_applied.load(std::sync::atomic::Ordering::Relaxed),
                "rl_cooldown_set": input.metrics.rl_cooldown_set.load(std::sync::atomic::Ordering::Relaxed),
                "deadletter_count": input.metrics.deadletter_count.load(std::sync::atomic::Ordering::Relaxed),
                "outq_len": input.metrics.outq_len.load(std::sync::atomic::Ordering::Relaxed),
                "wal_queue_len": input.metrics.wal_queue_len.load(std::sync::atomic::Ordering::Relaxed),
                "last_inbound_age_ms": input.metrics.last_inbound_age_ms.load(std::sync::atomic::Ordering::Relaxed),
                "inbound_frames": input.metrics.inbound_frames.load(std::sync::atomic::Ordering::Relaxed),
                "inbound_bytes": input.metrics.inbound_bytes.load(std::sync::atomic::Ordering::Relaxed),
                "outbound_frames": input.metrics.outbound_frames.load(std::sync::atomic::Ordering::Relaxed),
                "outbound_bytes": input.metrics.outbound_bytes.load(std::sync::atomic::Ordering::Relaxed),
                "decode_error": input.metrics.decode_error.load(std::sync::atomic::Ordering::Relaxed),
                "rl_wait_ms_total": input.metrics.rl_wait_ms_total.load(std::sync::atomic::Ordering::Relaxed),
                "wal_write_latency_ms_last": input.metrics.wal_write_latency_ms_last.load(std::sync::atomic::Ordering::Relaxed),
                "wal_write_latency_ms_max": input.metrics.wal_write_latency_ms_max.load(std::sync::atomic::Ordering::Relaxed)
            },
            "events_tail": events_tail,
            "rules_snapshot": input.rules_snapshot,
            "observability": {
                "metrics_prometheus_text": metrics_prom,
                "events_json": events_text
            },
        },
        "hub": {},
        "wal": {},
        "errors": [],
        "manifest": manifest
    })
}

fn emit_support_bundle_audit_hook(req: &ucel_diagnostics_core::DiagnosticsRequest) {
    tracing::info!(
        target: "ucel.transport.support_bundle",
        action = "build_support_bundle_archive",
        allow_deep = req.allow_deep,
        has_approval_id = req.approval_id.is_some(),
        "support bundle build requested"
    );
}

pub fn build_support_bundle_archive(
    registry: &ucel_diagnostics_core::DiagnosticsRegistry,
    req: &ucel_diagnostics_core::DiagnosticsRequest,
    limits: &crate::diagnostics::limits::BundleLimits,
) -> Result<crate::diagnostics::bundle::BuiltBundle, crate::diagnostics::limits::BundleBuildError> {
    emit_support_bundle_audit_hook(req);
    crate::diagnostics::bundle::build_support_bundle_tar_zst(registry, req, limits)
}
