use serde_json::json;
use ucel_transport::security::{redact_json_value, redact_kv_pairs, RedactionPolicy};

#[test]
fn redacts_sensitive_headers_and_query_pairs() {
    let policy = RedactionPolicy::default();
    let pairs = vec![
        ("Authorization".to_string(), "Bearer SECRET".to_string()),
        ("X-Api-Key".to_string(), "K".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
        ("symbol".to_string(), "BTCUSDT".to_string()),
    ];

    let red = redact_kv_pairs(&policy, pairs);
    let mut m = std::collections::HashMap::new();
    for (k, v) in red {
        m.insert(k, v);
    }

    assert_eq!(m.get("Authorization").unwrap(), "***REDACTED***");
    assert_eq!(m.get("X-Api-Key").unwrap(), "***REDACTED***");
    assert_eq!(m.get("Content-Type").unwrap(), "application/json");
    assert_eq!(m.get("symbol").unwrap(), "BTCUSDT");
}

#[test]
fn redacts_sensitive_json_keys_recursively() {
    let policy = RedactionPolicy::default();
    let mut v = json!({
        "ok": true,
        "auth": {
            "apiKey": "SECRET",
            "signature": "SIG",
            "nested": [{"token": "T"}]
        }
    });

    redact_json_value(&policy, &mut v);

    assert_eq!(v["auth"]["apiKey"], "***REDACTED***");
    assert_eq!(v["auth"]["signature"], "***REDACTED***");
    assert_eq!(v["auth"]["nested"][0]["token"], "***REDACTED***");
    assert_eq!(v["ok"], true);
}
