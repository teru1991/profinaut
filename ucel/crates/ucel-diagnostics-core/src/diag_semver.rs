use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum DiagSemverError {
    #[error("invalid diag_semver: {0}")]
    Invalid(String),
}

/// Diagnostic interface compatibility SemVer wrapper.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagSemver(String);

impl DiagSemver {
    pub fn parse(s: &str) -> Result<Self, DiagSemverError> {
        Version::parse(s).map_err(|e| DiagSemverError::Invalid(e.to_string()))?;
        if s.len() > 64 {
            return Err(DiagSemverError::Invalid("too long".into()));
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_version(&self) -> Version {
        // parse already validated; if someone deserialized an invalid string, validate again
        Version::parse(&self.0).expect("DiagSemver must be a valid semver string")
    }
}

impl fmt::Display for DiagSemver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
