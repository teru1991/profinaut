use std::fs;

#[test]
fn old_bundle_is_supported_or_warning() {
    let root = ucel_testkit::diagnostics::repo_root();
    let bundle: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/support_bundle/bundle_old_minor.json")).unwrap(),
    )
    .unwrap();
    let (_summary, compat, _drift) =
        ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root)
            .expect("analyze old");
    let status = compat.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert!(status == "Supported" || status == "SupportedWithWarnings");
}

#[test]
fn unsupported_major_is_rejected() {
    let status = ucel_diagnostics_analyzer::compat::evaluate_compatibility("2.0.0");
    assert_eq!(format!("{:?}", status), "Unsupported");
}
