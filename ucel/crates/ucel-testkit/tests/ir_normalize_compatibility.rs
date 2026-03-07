use serde_json::Value;
use ucel_testkit::ir_normalize::{fixture_path, load_text_fixture};

#[test]
fn expected_schema_version_is_v1() {
    let j = load_text_fixture(&fixture_path("html/minimal_notice.expected.normalized.json"));
    let v: Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["normalization_schema_version"]["major"], 1);
}
