use std::sync::atomic::Ordering;

use ucel_transport::diagnostics::support_bundle::{build_support_bundle, SupportBundleInput};
use ucel_transport::health::TransportHealth;
use ucel_transport::obs::{StabilityEventRing, TransportMetrics};

fn get_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut cur = root;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

#[test]
fn generated_support_bundle_matches_manifest_requirements() {
    let metrics = TransportMetrics::new();
    let events = StabilityEventRing::new(16);
    metrics.reconnect_attempts.fetch_add(1, Ordering::Relaxed);

    let bundle = build_support_bundle(SupportBundleInput {
        exchange_id: "bybit".to_string(),
        conn_id: "conn-1".to_string(),
        health: TransportHealth::healthy(),
        metrics,
        events,
        rules_snapshot: serde_json::json!({"mode":"test"}),
    });

    let manifest = bundle.get("manifest").expect("bundle manifest exists");
    assert_eq!(manifest.get("version").and_then(|v| v.as_u64()), Some(1));

    let required = manifest
        .get("required_paths")
        .and_then(|v| v.as_array())
        .expect("required_paths array");
    for req in required {
        let path = req.as_str().expect("required path str");
        assert!(
            get_path(&bundle, path).is_some(),
            "missing required path in generated bundle: {path}"
        );
    }

    let deny = manifest
        .get("deny_patterns")
        .and_then(|v| v.as_array())
        .expect("deny_patterns array");
    let rendered = serde_json::to_string(&bundle).expect("bundle json string");
    for pat in deny {
        let pat = pat.as_str().expect("pattern str");
        assert!(!rendered.contains(pat), "denied pattern leaked: {pat}");
    }
}
