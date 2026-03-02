use ucel_transport::security::{check_json_limits, JsonLimits};

#[test]
fn json_limits_allow_small() {
    let limits = JsonLimits {
        max_bytes: 64,
        max_depth: 4,
    };
    let s = br#"{"a":[{"b":1}]}"#;
    check_json_limits(s, limits).unwrap();
}

#[test]
fn json_limits_reject_depth() {
    let limits = JsonLimits {
        max_bytes: 1024,
        max_depth: 2,
    };
    let s = br#"{"a":{"b":{"c":1}}}"#;
    let err = check_json_limits(s, limits).unwrap_err();
    assert!(err.message.contains("depth"));
}

#[test]
fn json_limits_reject_bytes() {
    let limits = JsonLimits {
        max_bytes: 4,
        max_depth: 8,
    };
    let s = br#"{"a":1}"#;
    let err = check_json_limits(s, limits).unwrap_err();
    assert!(err.message.contains("too large"));
}
