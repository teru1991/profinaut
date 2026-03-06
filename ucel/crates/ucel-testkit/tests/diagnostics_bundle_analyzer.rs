use std::fs;

#[test]
fn analyzer_generates_summary_and_reports() {
    let root = ucel_testkit::diagnostics::repo_root();
    let fixture = root.join("fixtures/support_bundle/bundle_v1.json");
    let bundle: serde_json::Value = serde_json::from_slice(&fs::read(fixture).unwrap()).unwrap();

    let (summary, compat, drift) =
        ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root).expect("analyze");

    assert!(summary.get("coverage_hash").is_some());
    assert!(summary.get("runtime_capability_hash").is_some());
    assert!(compat.get("status").is_some());
    assert!(drift.get("findings").is_some());
}

#[test]
fn broken_bundle_is_error_not_panic() {
    let root = ucel_testkit::diagnostics::repo_root();
    let fixture = root.join("fixtures/support_bundle/bundle_broken.json");
    let bundle: serde_json::Value = serde_json::from_slice(&fs::read(fixture).unwrap()).unwrap();
    let err = ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("redaction") || msg.contains("manifest"));
}
