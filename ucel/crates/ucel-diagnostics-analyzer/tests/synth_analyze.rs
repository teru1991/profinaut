use std::{fs, path::PathBuf};
use ucel_diagnostics_analyzer::{analyze_tar_zst_bundle, synth::build_minimal_bundle_v1};

#[test]
fn synth_bundle_matches_expected_summary_fixture() {
    let synth = build_minimal_bundle_v1();
    let (_, summary) = analyze_tar_zst_bundle(synth.tar_zst_bytes).expect("analyze v1 synth bundle");

    let mut actual = serde_json::to_value(summary).expect("serialize summary");
    strip_dynamic_fields(&mut actual);

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/golden/support_bundle/v1/expected.summary.json");
    let mut expected: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture).expect("read expected fixture")).expect("parse fixture json");
    strip_dynamic_fields(&mut expected);

    assert_eq!(actual, expected);
}

fn strip_dynamic_fields(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        obj.remove("bundle_id");
        obj.remove("created_at_rfc3339");
        obj.remove("total_bytes");
    }
}
