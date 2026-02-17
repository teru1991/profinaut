//! Collector config (`collector.toml`) model, loader, and validator.

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
