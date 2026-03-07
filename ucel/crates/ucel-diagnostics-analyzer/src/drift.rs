use std::path::Path;
use ucel_core::DriftFinding;

pub fn detect_runbook_drift(repo_root: &Path) -> Vec<DriftFinding> {
    ucel_diagnostics_core::runbook::drift_findings_for_docs(
        repo_root,
        &["ucel/docs", "docs/specs/ucel", "docs/runbooks"],
    )
}
