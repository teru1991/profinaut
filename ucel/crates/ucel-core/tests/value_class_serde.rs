use serde::Deserialize;
use ucel_core::decimal::serde::{deserialize_decimal_execution, deserialize_decimal_observation};
use ucel_core::Decimal;

#[derive(Deserialize)]
struct ExecWire {
    #[serde(deserialize_with = "deserialize_decimal_execution")]
    v: Decimal,
}

#[derive(Deserialize)]
struct ObsWire {
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    v: Decimal,
}

#[test]
fn execution_rejects_zero_by_default() {
    let j = r#"{"v":"0"}"#;
    assert!(serde_json::from_str::<ExecWire>(j).is_err());
}

#[test]
fn observation_allows_zero() {
    let j = r#"{"v":"0"}"#;
    assert!(serde_json::from_str::<ObsWire>(j).is_ok());
}

#[test]
fn execution_rejects_negative() {
    let j = r#"{"v":"-1"}"#;
    assert!(serde_json::from_str::<ExecWire>(j).is_err());
}
