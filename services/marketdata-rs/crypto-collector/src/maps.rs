//! Maps loader and normalization helpers.
//!
//! Loads `symbol_map_file` (TOML) and `channel_map` (inline descriptor table),
//! then provides normalization functions that map raw exchange values to
//! canonical Profinaut values.

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MapsError {
    #[error("failed to read symbol map file '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse symbol map TOML: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Loaded and ready-to-use normalization maps.
#[derive(Debug, Clone, Default)]
pub struct NormalizationMaps {
    /// raw_symbol → canonical_symbol
    pub symbol_map: HashMap<String, String>,
    /// raw_channel → canonical_channel
    pub channel_map: HashMap<String, String>,
}

impl NormalizationMaps {
    /// Normalize a raw symbol using the symbol map.
    /// Returns the mapped value if present, else the raw value unchanged.
    pub fn normalize_symbol<'a>(&'a self, raw: &'a str) -> &'a str {
        self.symbol_map.get(raw).map(|s| s.as_str()).unwrap_or(raw)
    }

    /// Normalize a raw channel using the channel map.
    /// Returns the mapped value if present, else the raw value unchanged.
    pub fn normalize_channel<'a>(&'a self, raw: &'a str) -> &'a str {
        self.channel_map.get(raw).map(|s| s.as_str()).unwrap_or(raw)
    }
}

/// Load a symbol map from a TOML file.
///
/// Expected format:
/// ```toml
/// [symbols]
/// BTCUSDT = "BTC/USDT"
/// ETHUSDT = "ETH/USDT"
/// ```
pub fn load_symbol_map(path: &Path) -> Result<HashMap<String, String>, MapsError> {
    let content = std::fs::read_to_string(path).map_err(|e| MapsError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    parse_symbol_map(&content)
}

/// Parse a symbol map from a TOML string.
fn parse_symbol_map(content: &str) -> Result<HashMap<String, String>, MapsError> {
    let table: toml::Value = toml::from_str(content)?;
    let mut map = HashMap::new();

    if let Some(symbols) = table.get("symbols").and_then(|v| v.as_table()) {
        for (k, v) in symbols {
            if let Some(s) = v.as_str() {
                map.insert(k.clone(), s.to_string());
            }
        }
    }

    Ok(map)
}

/// Build a channel map from a descriptor's inline `channel_map` table.
pub fn build_channel_map(table: &toml::value::Table) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in table {
        if let Some(s) = v.as_str() {
            map.insert(k.clone(), s.to_string());
        }
    }
    map
}

/// Build `NormalizationMaps` from descriptor maps section, resolving paths
/// relative to `base_dir`.
pub fn load_maps(
    symbol_map_file: Option<&str>,
    channel_map_table: Option<&toml::value::Table>,
    base_dir: &Path,
) -> Result<NormalizationMaps, MapsError> {
    let symbol_map = match symbol_map_file {
        Some(path_str) => {
            let path = if Path::new(path_str).is_absolute() {
                Path::new(path_str).to_path_buf()
            } else {
                base_dir.join(path_str)
            };
            load_symbol_map(&path)?
        }
        None => HashMap::new(),
    };

    let channel_map = match channel_map_table {
        Some(table) => build_channel_map(table),
        None => HashMap::new(),
    };

    Ok(NormalizationMaps {
        symbol_map,
        channel_map,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_symbol_map_toml() {
        let content = r#"
[symbols]
BTCUSDT = "BTC/USDT"
ETHUSDT = "ETH/USDT"
"#;
        let map = parse_symbol_map(content).unwrap();
        assert_eq!(map.get("BTCUSDT"), Some(&"BTC/USDT".to_string()));
        assert_eq!(map.get("ETHUSDT"), Some(&"ETH/USDT".to_string()));
        assert_eq!(map.get("XRPUSDT"), None);
    }

    #[test]
    fn build_channel_map_from_table() {
        let mut table = toml::value::Table::new();
        table.insert(
            "trade".to_string(),
            toml::Value::String("trades".to_string()),
        );
        table.insert(
            "book".to_string(),
            toml::Value::String("orderbook_l2".to_string()),
        );

        let map = build_channel_map(&table);
        assert_eq!(map.get("trade"), Some(&"trades".to_string()));
        assert_eq!(map.get("book"), Some(&"orderbook_l2".to_string()));
    }

    #[test]
    fn normalize_symbol_mapped() {
        let maps = NormalizationMaps {
            symbol_map: HashMap::from([("BTCUSDT".to_string(), "BTC/USDT".to_string())]),
            channel_map: HashMap::new(),
        };
        assert_eq!(maps.normalize_symbol("BTCUSDT"), "BTC/USDT");
    }

    #[test]
    fn normalize_symbol_passthrough() {
        let maps = NormalizationMaps::default();
        assert_eq!(maps.normalize_symbol("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn normalize_channel_mapped() {
        let maps = NormalizationMaps {
            symbol_map: HashMap::new(),
            channel_map: HashMap::from([("trade".to_string(), "trades".to_string())]),
        };
        assert_eq!(maps.normalize_channel("trade"), "trades");
    }

    #[test]
    fn normalize_channel_passthrough() {
        let maps = NormalizationMaps::default();
        assert_eq!(maps.normalize_channel("raw_channel"), "raw_channel");
    }

    #[test]
    fn load_maps_no_files() {
        let maps = load_maps(None, None, Path::new(".")).unwrap();
        assert!(maps.symbol_map.is_empty());
        assert!(maps.channel_map.is_empty());
    }

    #[test]
    fn load_maps_missing_symbol_file_errors() {
        let err = load_maps(Some("nonexistent.toml"), None, Path::new(".")).unwrap_err();
        assert!(err.to_string().contains("nonexistent.toml"));
    }
}
