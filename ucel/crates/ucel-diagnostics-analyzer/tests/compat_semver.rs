use ucel_diagnostics_analyzer::{
    analyze_tar_zst_bundle,
    summary::SummaryBuildError,
    synth::{build_minimal_bundle_v1, build_minimal_bundle_v2_major2},
};

#[test]
fn accepts_supported_semver_major_v1() {
    let synth = build_minimal_bundle_v1();
    analyze_tar_zst_bundle(synth.tar_zst_bytes).expect("v1 should be supported");
}

#[test]
fn rejects_unsupported_semver_major_v2() {
    let synth = build_minimal_bundle_v2_major2();
    let err = analyze_tar_zst_bundle(synth.tar_zst_bytes).expect_err("v2 major must be rejected");

    assert!(matches!(err, SummaryBuildError::Unsupported(v) if v == "2.0.0"));
}
