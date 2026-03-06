use ucel_core::DriftFinding;
use std::path::Path;

pub fn detect_runbook_drift(repo_root: &Path) -> Vec<DriftFinding> {
    ucel_diagnostics_core::runbook::drift_findings_for_docs(
        repo_root,
        &["ucel/docs", "docs/specs/ucel", "docs/runbooks"],
    )
}
