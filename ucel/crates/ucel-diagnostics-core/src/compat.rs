use semver::Version;
use ucel_core::{CompatibilityStatus, DiagnosticsSemver};

pub fn compatibility_for(analyzer_supported_major: u64, bundle: &DiagnosticsSemver) -> CompatibilityStatus {
    match Version::parse(&bundle.0) {
        Ok(v) if v.major == analyzer_supported_major => {
            if v.minor == 0 && v.patch == 0 {
                CompatibilityStatus::Supported
            } else {
                CompatibilityStatus::SupportedWithWarnings
            }
        }
        _ => CompatibilityStatus::Unsupported,
    }
}
