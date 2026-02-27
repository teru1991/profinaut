use crate::error::{SdkError, SdkResult};
use crate::secrets::SecretString;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use ucel_registry::hub::HubConfig;

/// Optional config file schema (TOML).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdkConfigFile {
    #[serde(default)]
    pub hub: HubConfigFile,
    #[serde(default)]
    pub runtime: RuntimeConfigFile,
    #[serde(default)]
    pub auth: AuthConfigFile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HubConfigFile {
    pub request_timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub base_backoff_ms: Option<u64>,
    pub max_backoff_ms: Option<u64>,
    pub ws_buffer: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfigFile {
    pub repo_root: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfigFile {
    /// Optional per-exchange API keys (if you later extend hub/rest/ws to use it).
    /// Keep as redacted secrets.
    #[serde(default)]
    pub by_exchange: std::collections::BTreeMap<String, AuthPairFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthPairFile {
    pub api_key: Option<SecretString>,
    pub api_secret: Option<SecretString>,
}

/// In-memory validated SDK config (canonical).
#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub hub: HubConfig,
    pub repo_root: PathBuf,
    pub run_id: String,

    /// Auth map (redacted). Not yet wired into hub, but SSOT here prevents ad-hoc handling.
    pub auth_by_exchange: std::collections::BTreeMap<String, AuthPair>,
}

#[derive(Debug, Clone, Default)]
pub struct AuthPair {
    pub api_key: Option<SecretString>,
    pub api_secret: Option<SecretString>,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            hub: HubConfig::default(),
            repo_root: PathBuf::from("."),
            run_id: "run-local".to_string(),
            auth_by_exchange: Default::default(),
        }
    }
}

impl SdkConfig {
    /// Load config from (optional) toml file + env overrides.
    /// Env overrides take precedence.
    ///
    /// Supported env:
    /// - UCEL_REPO_ROOT
    /// - UCEL_RUN_ID
    /// - UCEL_HUB_REQUEST_TIMEOUT_MS
    /// - UCEL_HUB_MAX_RETRIES
    /// - UCEL_HUB_BASE_BACKOFF_MS
    /// - UCEL_HUB_MAX_BACKOFF_MS
    /// - UCEL_HUB_WS_BUFFER
    pub fn load(path: Option<&Path>) -> SdkResult<Self> {
        let mut cfg = Self::default();

        if let Some(p) = path {
            let raw = fs::read_to_string(p)?;
            let file_cfg: SdkConfigFile = toml::from_str(&raw)?;
            cfg.apply_file(file_cfg)?;
        }

        cfg.apply_env()?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn apply_file(&mut self, file_cfg: SdkConfigFile) -> SdkResult<()> {
        // hub
        if let Some(v) = file_cfg.hub.request_timeout_ms {
            self.hub.request_timeout = Duration::from_millis(v);
        }
        if let Some(v) = file_cfg.hub.max_retries {
            self.hub.max_retries = v;
        }
        if let Some(v) = file_cfg.hub.base_backoff_ms {
            self.hub.base_backoff_ms = v;
        }
        if let Some(v) = file_cfg.hub.max_backoff_ms {
            self.hub.max_backoff_ms = v;
        }
        if let Some(v) = file_cfg.hub.ws_buffer {
            self.hub.ws_buffer = v;
        }

        // runtime
        if let Some(rr) = file_cfg.runtime.repo_root {
            self.repo_root = PathBuf::from(rr);
        }
        if let Some(rid) = file_cfg.runtime.run_id {
            self.run_id = rid;
        }

        // auth
        for (k, v) in file_cfg.auth.by_exchange {
            self.auth_by_exchange.insert(
                k,
                AuthPair {
                    api_key: v.api_key,
                    api_secret: v.api_secret,
                },
            );
        }

        Ok(())
    }

    fn apply_env(&mut self) -> SdkResult<()> {
        if let Ok(v) = std::env::var("UCEL_REPO_ROOT") {
            if !v.trim().is_empty() {
                self.repo_root = PathBuf::from(v);
            }
        }
        if let Ok(v) = std::env::var("UCEL_RUN_ID") {
            if !v.trim().is_empty() {
                self.run_id = v;
            }
        }

        // hub overrides
        if let Ok(v) = std::env::var("UCEL_HUB_REQUEST_TIMEOUT_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                self.hub.request_timeout = Duration::from_millis(ms);
            }
        }
        if let Ok(v) = std::env::var("UCEL_HUB_MAX_RETRIES") {
            if let Ok(n) = v.parse::<u32>() {
                self.hub.max_retries = n;
            }
        }
        if let Ok(v) = std::env::var("UCEL_HUB_BASE_BACKOFF_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                self.hub.base_backoff_ms = ms;
            }
        }
        if let Ok(v) = std::env::var("UCEL_HUB_MAX_BACKOFF_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                self.hub.max_backoff_ms = ms;
            }
        }
        if let Ok(v) = std::env::var("UCEL_HUB_WS_BUFFER") {
            if let Ok(n) = v.parse::<usize>() {
                self.hub.ws_buffer = n;
            }
        }
        Ok(())
    }

    fn validate(&self) -> SdkResult<()> {
        if self.run_id.trim().is_empty() {
            return Err(SdkError::Config("run_id empty".to_string()));
        }
        // repo_root existence check (soft: require directory)
        if !self.repo_root.exists() {
            return Err(SdkError::Config(format!(
                "repo_root does not exist: {}",
                self.repo_root.display()
            )));
        }
        if !self.repo_root.is_dir() {
            return Err(SdkError::Config(format!(
                "repo_root is not a directory: {}",
                self.repo_root.display()
            )));
        }
        // safety bounds
        if self.hub.max_retries > 20 {
            return Err(SdkError::Config(
                "hub.max_retries too large (>20)".to_string(),
            ));
        }
        if self.hub.ws_buffer < 16 {
            return Err(SdkError::Config(
                "hub.ws_buffer too small (<16)".to_string(),
            ));
        }
        Ok(())
    }

    /// Save current config to a TOML file (secrets are redacted but structurally present).
    pub fn save_redacted(&self, path: &Path) -> SdkResult<()> {
        // write minimal file form
        let mut file = SdkConfigFile::default();
        file.runtime.repo_root = Some(self.repo_root.to_string_lossy().to_string());
        file.runtime.run_id = Some(self.run_id.clone());

        file.hub.request_timeout_ms = Some(self.hub.request_timeout.as_millis() as u64);
        file.hub.max_retries = Some(self.hub.max_retries);
        file.hub.base_backoff_ms = Some(self.hub.base_backoff_ms);
        file.hub.max_backoff_ms = Some(self.hub.max_backoff_ms);
        file.hub.ws_buffer = Some(self.hub.ws_buffer);

        for (k, v) in &self.auth_by_exchange {
            file.auth.by_exchange.insert(
                k.clone(),
                AuthPairFile {
                    api_key: v.api_key.clone(),
                    api_secret: v.api_secret.clone(),
                },
            );
        }

        let raw = toml::to_string_pretty(&file)
            .map_err(|e| SdkError::Config(format!("toml encode: {e}")))?;
        fs::write(path, raw)?;
        Ok(())
    }
}
