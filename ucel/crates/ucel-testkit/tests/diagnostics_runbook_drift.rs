#[test]
fn runbook_drift_checker_runs_and_detects_missing_docs_path() {
    let root = ucel_testkit::diagnostics::repo_root();
    let findings = ucel_diagnostics_core::runbook::drift_findings_for_docs(
        &root,
        &["docs/specs/ucel", "docs/definitely_missing_file.md"],
    );
    assert!(findings.iter().any(|f| f.kind == "path_missing"));
}
