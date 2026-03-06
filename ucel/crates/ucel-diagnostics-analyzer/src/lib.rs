pub mod analyze;
pub mod compat;
pub mod drift;
pub mod read;
pub mod summary;
pub mod synth;

pub use analyze::{analyze_support_bundle_value, AnalyzeError};
pub use read::{BundleReadError, BundleReader, ManifestFile, ParsedManifest};
pub use summary::{analyze_tar_zst_bundle, AnalyzerSummary, SummaryBuildError};

/// Supported diag_semver major for analyzer compatibility.
pub const SUPPORTED_DIAG_SEMVER_MAJOR: u64 = 1;
