use serde::{Deserialize, Serialize};
use std::fmt;

/// Secret string that redacts on Debug/Display.
/// Use `.expose()` only at the last moment when needed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretString(String);

impl SecretString {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }

    /// Expose underlying secret. Use sparingly.
    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(**redacted**)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("**redacted**")
    }
}
