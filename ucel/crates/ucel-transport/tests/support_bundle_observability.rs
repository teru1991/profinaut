use std::sync::atomic::Ordering;

use ucel_transport::diagnostics::support_bundle::{build_support_bundle, SupportBundleInput};
use ucel_transport::health::TransportHealth;
use ucel_transport::obs::{StabilityEventRing, TransportMetrics};

#[test]
fn support_bundle_contains_prom_and_event_context() {
    let metrics = TransportMetrics::new();
    let events = StabilityEventRing::new(64);

    metrics.reconnect_attempts.fetch_add(2, Ordering::Relaxed);
    metrics.on_inbound(128, 10_000);
    events.push_required(
        "reconnect_attempt",
        serde_json::json!({"attempt": 1}),
        "gmocoin",
        "conn-1",
        "run-1",
        "ws_connection",
        "*",
    );

    let health = TransportHealth::healthy();

    let bundle = build_support_bundle(SupportBundleInput {
        exchange_id: "gmocoin".into(),
        conn_id: "conn-1".into(),
        health,
        metrics,
        events,
        rules_snapshot: serde_json::json!({"ok": true}),
    });

    let prom = bundle["observability"]["metrics_prometheus_text"]
        .as_str()
        .unwrap();
    assert!(prom.contains("ucel_transport_reconnect_attempts_total 2"));

    let events_json = bundle["observability"]["events_json"].as_str().unwrap();
    assert!(events_json.contains("run-1"));
    assert!(events_json.contains("ws_connection"));
}
