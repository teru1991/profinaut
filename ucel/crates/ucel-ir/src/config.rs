use crate::errors::{UcelIrError, UcelIrErrorKind};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcelIrConfig {
    pub http: HttpConfig,
    pub raw_storage_root: String,
    pub checkpoint_path: String,
}

impl UcelIrConfig {
    pub fn validate(&self) -> Result<(), UcelIrError> {
        self.http.validate()?;
        if self.raw_storage_root.trim().is_empty() {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "raw_storage_root must not be empty",
            ));
        }
        if self.checkpoint_path.trim().is_empty() {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "checkpoint_path must not be empty",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub user_agent: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub rate_limit_per_sec: u32,
}

impl HttpConfig {
    pub fn validate(&self) -> Result<(), UcelIrError> {
        if self.user_agent.trim().is_empty() {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "http.user_agent is required",
            ));
        }
        if self.timeout_ms == 0 || self.base_backoff_ms == 0 || self.rate_limit_per_sec == 0 {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "http timeout/backoff/rate_limit values must be > 0",
            ));
        }
        Ok(())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}
