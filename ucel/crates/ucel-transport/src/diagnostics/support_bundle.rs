use crate::health::TransportHealth;
use crate::obs::{StabilityEventRing, TransportMetrics};

#[derive(Debug, Clone)]
pub struct SupportBundleInput {
    pub exchange_id: String,
    pub conn_id: String,
    pub health: TransportHealth,
    pub metrics: std::sync::Arc<TransportMetrics>,
    pub events: std::sync::Arc<StabilityEventRing>,
    pub rules_snapshot: serde_json::Value,
}

pub fn build_support_bundle(input: SupportBundleInput) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "exchange_id": input.exchange_id,
        "conn_id": input.conn_id,
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
            "last_inbound_age_ms": input.metrics.last_inbound_age_ms.load(std::sync::atomic::Ordering::Relaxed)
        },
        "events_tail": input.events.snapshot(),
        "rules_snapshot": input.rules_snapshot
    })
}
