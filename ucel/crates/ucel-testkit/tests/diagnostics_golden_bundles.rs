use std::fs;

#[test]
fn golden_bundle_summary_matches_expected_fixture() {
    let root = ucel_testkit::diagnostics::repo_root();
    let bundle: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/support_bundle/bundle_v1.json")).unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/support_bundle/expected_summary_v1.json")).unwrap(),
    )
    .unwrap();

    let (summary, _compat, _drift) =
        ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root).expect("analyze golden");
    assert_eq!(summary, expected);
}

#[test]
fn broken_bundle_fixture_is_expected_to_fail() {
    let root = ucel_testkit::diagnostics::repo_root();
    let bundle: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/support_bundle/bundle_broken.json")).unwrap(),
    )
    .unwrap();

    assert!(ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root).is_err());
}
