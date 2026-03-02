use crate::obs::catalog::{find, MetricKind};
use crate::obs::TransportMetrics;

/// Encode `TransportMetrics` into Prometheus text exposition format.
/// Labels are intentionally none at Step1 (stable base). Step2/3 may add labels carefully.
///
/// NOTE:
/// - Counters end with `_total`
/// - Gauges are plain
pub fn encode_prometheus_text(m: &TransportMetrics) -> String {
    fn push(out: &mut String, name: &str, kind: MetricKind, value: i128) {
        if let Some(def) = find(name) {
            out.push_str("# HELP ");
            out.push_str(def.name);
            out.push(' ');
            out.push_str(def.help);
            out.push('\n');

            out.push_str("# TYPE ");
            out.push_str(def.name);
            out.push(' ');
            out.push_str(match kind {
                MetricKind::Counter => "counter",
                MetricKind::Gauge => "gauge",
            });
            out.push('\n');

            out.push_str(def.name);
            out.push(' ');
            out.push_str(&value.to_string());
            out.push('\n');
        } else {
            out.push_str("# HELP ucel_transport_catalog_miss Catalog missing metric definition.\n");
            out.push_str("# TYPE ucel_transport_catalog_miss counter\n");
            out.push_str("ucel_transport_catalog_miss 1\n");
        }
    }

    use std::sync::atomic::Ordering;

    let mut out = String::new();

    push(
        &mut out,
        "ucel_transport_reconnect_attempts_total",
        MetricKind::Counter,
        m.reconnect_attempts.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_reconnect_success_total",
        MetricKind::Counter,
        m.reconnect_success.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_reconnect_failure_total",
        MetricKind::Counter,
        m.reconnect_failure.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_breaker_open_total",
        MetricKind::Counter,
        m.breaker_open.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_stale_requeued_total",
        MetricKind::Counter,
        m.stale_requeued.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_outq_dropped_total",
        MetricKind::Counter,
        m.outq_dropped.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_outq_spilled_total",
        MetricKind::Counter,
        m.outq_spilled.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_rl_penalty_applied_total",
        MetricKind::Counter,
        m.rl_penalty_applied.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_rl_cooldown_set_total",
        MetricKind::Counter,
        m.rl_cooldown_set.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_deadletter_total",
        MetricKind::Counter,
        m.deadletter_count.load(Ordering::Relaxed) as i128,
    );

    push(
        &mut out,
        "ucel_transport_outq_len",
        MetricKind::Gauge,
        m.outq_len.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_wal_queue_len",
        MetricKind::Gauge,
        m.wal_queue_len.load(Ordering::Relaxed) as i128,
    );
    push(
        &mut out,
        "ucel_transport_last_inbound_age_ms",
        MetricKind::Gauge,
        m.last_inbound_age_ms.load(Ordering::Relaxed) as i128,
    );

    out
}
