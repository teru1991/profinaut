//! Collector config (`collector.toml`) model, loader, and validator.
//!
//! Task D adds an optional `[persistence]` section.  Existing configs that
//! omit `[persistence]` continue to work (all fields default to disabled).

use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse config TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("config validation failed:\n{}", format_errors(.0))]
    Validation(Vec<String>),
}

fn format_errors(errors: &[String]) -> String {
    errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {}. {}", i + 1, e))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CollectorConfig {
    pub run: RunConfig,
    #[serde(rename = "exchange")]
    pub exchanges: Vec<ExchangeInstance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunConfig {
    pub http_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ExchangeInstance {
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub descriptor_path: String,
    pub symbols: Vec<String>,
    pub channels: Vec<String>,
    #[serde(default)]
    pub overrides: Option<toml::Value>,
}

fn default_enabled() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Persistence config (Task D)
// ---------------------------------------------------------------------------

/// Top-level persistence configuration (maps to `[persistence]` in TOML).
/// All sub-sections default to disabled; existing configs remain valid.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PersistenceConfig {
    /// MongoDB connection URI.
    #[serde(default = "default_mongo_uri")]
    pub mongo_uri: String,
    /// MongoDB database name.
    #[serde(default = "default_mongo_database")]
    pub mongo_database: String,
    /// MongoDB collection name for raw envelopes.
    #[serde(default = "default_mongo_collection")]
    pub mongo_collection: String,
    /// Max retries per batch before declaring MongoUnavailable.
    #[serde(default = "default_mongo_max_retries")]
    pub mongo_max_retries: u32,
    /// Base delay (ms) for exponential retry backoff.
    #[serde(default = "default_mongo_retry_base_ms")]
    pub mongo_retry_base_ms: u64,
    /// Consecutive batch failures before transitioning to Degraded state.
    #[serde(default = "default_consecutive_failures_for_degraded")]
    pub mongo_consecutive_failures_for_degraded: u32,

    #[serde(default)]
    pub spool: SpoolConfigToml,

    #[serde(default)]
    pub dedup: DedupConfigToml,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpoolConfigToml {
    /// Enable the durable spool (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Directory for spool segment files.
    #[serde(default = "default_spool_dir")]
    pub dir: String,
    /// Maximum size of a single spool segment in MiB (default: 64).
    #[serde(default = "default_max_segment_mb")]
    pub max_segment_mb: u64,
    /// Maximum total spool size in MiB (default: 1024).
    #[serde(default = "default_max_total_mb")]
    pub max_total_mb: u64,
    /// Policy when spool is full: "drop_ticker_depth_keep_trade" | "drop_all" | "block".
    #[serde(default = "default_on_full")]
    pub on_full: String,
}

impl Default for SpoolConfigToml {
    fn default() -> Self {
        Self {
            enabled: false,
            dir: default_spool_dir(),
            max_segment_mb: default_max_segment_mb(),
            max_total_mb: default_max_total_mb(),
            on_full: default_on_full(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DedupConfigToml {
    /// Enable the dedup window (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Seconds to keep a key before eviction (default: 300).
    #[serde(default = "default_dedup_window_seconds")]
    pub window_seconds: u64,
    /// Max number of live keys before evicting oldest (default: 100000).
    #[serde(default = "default_dedup_max_keys")]
    pub max_keys: usize,
}

impl Default for DedupConfigToml {
    fn default() -> Self {
        Self {
            enabled: false,
            window_seconds: default_dedup_window_seconds(),
            max_keys: default_dedup_max_keys(),
        }
    }
}

fn default_mongo_uri() -> String {
    "mongodb://localhost:27017".to_string()
}
fn default_mongo_database() -> String {
    "market_data".to_string()
}
fn default_mongo_collection() -> String {
    "crypto_envelopes".to_string()
}
fn default_mongo_max_retries() -> u32 {
    3
}
fn default_mongo_retry_base_ms() -> u64 {
    100
}
fn default_consecutive_failures_for_degraded() -> u32 {
    3
}
fn default_spool_dir() -> String {
    "/tmp/crypto-spool".to_string()
}
fn default_max_segment_mb() -> u64 {
    64
}
fn default_max_total_mb() -> u64 {
    1024
}
fn default_on_full() -> String {
    "drop_ticker_depth_keep_trade".to_string()
}
fn default_dedup_window_seconds() -> u64 {
    300
}
fn default_dedup_max_keys() -> usize {
    100_000
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load and validate a collector config from a TOML file path.
pub fn load_config(path: &Path) -> Result<CollectorConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let config: CollectorConfig = toml::from_str(&content)?;
    validate_config(&config)?;
    Ok(config)
}

/// Parse and validate a collector config from a TOML string.
#[allow(dead_code)]
pub fn parse_config(content: &str) -> Result<CollectorConfig, ConfigError> {
    let config: CollectorConfig = toml::from_str(content)?;
    validate_config(&config)?;
    Ok(config)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_config(config: &CollectorConfig) -> Result<(), ConfigError> {
    let mut errors: Vec<String> = Vec::new();

    // http_port: must be > 0 (u16 already guarantees <= 65535)
    if config.run.http_port == 0 {
        errors.push("run.http_port must be > 0".to_string());
    }

    // log_level: basic sanity check
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.run.log_level.to_lowercase().as_str()) {
        errors.push(format!(
            "run.log_level '{}' is not a valid level (expected one of: {})",
            config.run.log_level,
            valid_levels.join(", ")
        ));
    }

    // Exchange instances
    if config.exchanges.is_empty() {
        errors.push("at least one [[exchange]] instance must be defined".to_string());
    }

    // Uniqueness of exchange instance names
    let mut seen_names = HashSet::new();
    for inst in &config.exchanges {
        if !seen_names.insert(&inst.name) {
            errors.push(format!("exchange '{}': duplicate instance name", inst.name));
        }

        // Per-instance validation
        validate_exchange_instance(inst, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Validation(errors))
    }
}

fn validate_exchange_instance(inst: &ExchangeInstance, errors: &mut Vec<String>) {
    let ctx = &inst.name;

    if inst.name.is_empty() {
        errors.push("exchange instance has empty name".to_string());
    }

    if inst.descriptor_path.is_empty() {
        errors.push(format!("exchange '{ctx}': descriptor_path is empty"));
    }

    // For enabled instances, symbols and channels must be non-empty
    if inst.enabled {
        if inst.symbols.is_empty() {
            errors.push(format!(
                "exchange '{ctx}': symbols must be non-empty for enabled instances"
            ));
        }
        if inst.channels.is_empty() {
            errors.push(format!(
                "exchange '{ctx}': channels must be non-empty for enabled instances"
            ));
        }
    }

    // Even disabled instances: validate shape (non-empty arrays are warnings, not errors)
    // descriptor_path existence is checked at load time (in the service), not here.
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_toml() -> &'static str {
        r#"
[run]
http_port = 8080
log_level = "info"

[[exchange]]
name = "binance"
enabled = true
descriptor_path = "exchanges/binance_v1_4.toml"
symbols = ["BTC/USDT", "ETH/USDT"]
channels = ["trades", "orderbook"]

[[exchange]]
name = "kraken"
enabled = false
descriptor_path = "exchanges/kraken_v1_4.toml"
symbols = []
channels = []
"#
    }

    #[test]
    fn parse_valid_config() {
        let cfg = parse_config(valid_toml()).unwrap();
        assert_eq!(cfg.run.http_port, 8080);
        assert_eq!(cfg.exchanges.len(), 2);
        assert_eq!(cfg.exchanges[0].name, "binance");
        assert!(cfg.exchanges[0].enabled);
        assert!(!cfg.exchanges[1].enabled);
    }

    #[test]
    fn reject_duplicate_names() {
        let toml = r#"
[run]
http_port = 8080
log_level = "info"

[[exchange]]
name = "binance"
enabled = true
descriptor_path = "exchanges/binance.toml"
symbols = ["BTC/USDT"]
channels = ["trades"]

[[exchange]]
name = "binance"
enabled = true
descriptor_path = "exchanges/binance2.toml"
symbols = ["ETH/USDT"]
channels = ["trades"]
"#;
        let err = parse_config(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate instance name"), "got: {msg}");
    }

    #[test]
    fn reject_zero_port() {
        let toml = r#"
[run]
http_port = 0
log_level = "info"

[[exchange]]
name = "test"
descriptor_path = "test.toml"
symbols = ["X"]
channels = ["Y"]
"#;
        let err = parse_config(toml).unwrap_err();
        assert!(err.to_string().contains("http_port must be > 0"));
    }

    #[test]
    fn reject_empty_symbols_for_enabled() {
        let toml = r#"
[run]
http_port = 8080
log_level = "info"

[[exchange]]
name = "test"
enabled = true
descriptor_path = "test.toml"
symbols = []
channels = ["trades"]
"#;
        let err = parse_config(toml).unwrap_err();
        assert!(err.to_string().contains("symbols must be non-empty"));
    }

    #[test]
    fn reject_invalid_log_level() {
        let toml = r#"
[run]
http_port = 8080
log_level = "banana"

[[exchange]]
name = "test"
descriptor_path = "test.toml"
symbols = ["X"]
channels = ["Y"]
"#;
        let err = parse_config(toml).unwrap_err();
        assert!(err.to_string().contains("not a valid level"));
    }

    #[test]
    fn allow_disabled_with_empty_symbols() {
        let toml = r#"
[run]
http_port = 8080
log_level = "info"

[[exchange]]
name = "test"
enabled = false
descriptor_path = "test.toml"
symbols = []
channels = []
"#;
        // Disabled instances with empty symbols/channels are allowed
        let cfg = parse_config(toml).unwrap();
        assert!(!cfg.exchanges[0].enabled);
    }
}
