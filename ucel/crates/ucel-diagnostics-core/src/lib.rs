pub mod diag_semver;
pub mod provider;
pub mod registry;

pub use diag_semver::{DiagSemver, DiagSemverError};
pub use provider::{Contribution, ContributionContent, ContributionKind, DiagnosticsProvider, DiagnosticsRequest, ProviderMeta};
pub use registry::{DiagnosticsRegistry, RegistryError, RegistryLimits};

/// Fixed, repo-wide diagnostic interface SemVer.
/// NOTE: This is the *diagnostics surface* compatibility version (Y spec: diag_semver),
/// not the crate version. Bump policy is defined in docs/specs/system/Y_*.
/// Keep this value stable; MAJOR bumps should be avoided and gated by compat tests (Task4).
pub const DIAG_SEMVER_STR: &str = "1.0.0";

pub fn diag_semver() -> DiagSemver {
    // safe unwrap because const is controlled here
    DiagSemver::parse(DIAG_SEMVER_STR).expect("DIAG_SEMVER_STR must be valid semver")
}
