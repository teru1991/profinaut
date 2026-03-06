use std::collections::BTreeMap;

use ucel_transport::redaction::{
    redact_body_map, redact_headers, redact_query, redact_sign_input_preview, RedactionPolicy,
};

#[test]
fn redaction_masks_sensitive_keys_case_insensitively() {
    let mut headers = BTreeMap::new();
    headers.insert("Authorization".into(), "Bearer real-token".into());
    headers.insert("X-API-KEY".into(), "real-key".into());
    headers.insert("Content-Type".into(), "application/json".into());

    let redacted = redact_headers(&RedactionPolicy::default(), &headers);
    assert_eq!(redacted.get("Authorization").unwrap(), "***redacted***");
    assert_eq!(redacted.get("X-API-KEY").unwrap(), "***redacted***");
    assert_eq!(redacted.get("Content-Type").unwrap(), "application/json");
}

#[test]
fn redaction_masks_query_and_body_without_leaking_values() {
    let mut query = BTreeMap::new();
    query.insert("signature".into(), "super-secret-signature".into());
    query.insert("symbol".into(), "BTC_JPY".into());

    let mut body = BTreeMap::new();
    body.insert("passphrase".into(), "passphrase-value".into());
    body.insert("session_token".into(), "session-token-value".into());

    let policy = RedactionPolicy::default();
    let rq = redact_query(&policy, &query);
    let rb = redact_body_map(&policy, &body);

    assert_eq!(rq.get("signature").unwrap(), "***redacted***");
    assert_eq!(rq.get("symbol").unwrap(), "BTC_JPY");
    assert_eq!(rb.get("passphrase").unwrap(), "***redacted***");
    assert_eq!(rb.get("session_token").unwrap(), "***redacted***");
    assert!(!format!("{:?}{:?}", rq, rb).contains("super-secret"));
}

#[test]
fn sign_input_preview_is_redacted() {
    let policy = RedactionPolicy::default();
    let preview = "method=POST api_secret=top-secret signature=abc";
    let redacted = redact_sign_input_preview(&policy, preview);
    assert_eq!(redacted, "***redacted***");
    assert!(!redacted.contains("top-secret"));
}
