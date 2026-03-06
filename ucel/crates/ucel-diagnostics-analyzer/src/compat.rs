use ucel_core::{CompatibilityStatus, DiagnosticsSemver};

pub fn evaluate_compatibility(diag_semver: &str) -> CompatibilityStatus {
    let sem = DiagnosticsSemver(diag_semver.to_string());
    ucel_diagnostics_core::compatibility_for(crate::SUPPORTED_DIAG_SEMVER_MAJOR, &sem)
}
