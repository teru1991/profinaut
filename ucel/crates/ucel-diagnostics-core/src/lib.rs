pub mod bundle;
pub mod compat;
pub mod diag_semver;
pub mod hash;
pub mod manifest;
pub mod provider;
pub mod redaction;
pub mod registry;
pub mod runbook;
pub mod summary;

pub use compat::compatibility_for;
pub use diag_semver::{DiagSemver, DiagSemverError};
pub use hash::{default_hash_set, hash_paths, HashError};
pub use manifest::{manifest_to_pretty_json, parse_manifest_bytes, ManifestError};
pub use provider::{
    Contribution, ContributionContent, ContributionKind, DiagnosticsProvider, DiagnosticsRequest,
    ProviderMeta,
};
pub use registry::{DiagnosticsRegistry, RegistryError, RegistryLimits};

/// Fixed, repo-wide diagnostic interface SemVer.
pub const DIAG_SEMVER_STR: &str = "1.1.0";

pub fn diag_semver() -> DiagSemver {
    DiagSemver::parse(DIAG_SEMVER_STR).expect("DIAG_SEMVER_STR must be valid semver")
}
