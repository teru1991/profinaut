//! B5 — Maps Loader + Normalization.
//!
//! Loads symbol and channel mapping tables for descriptor-driven normalization.
//!
//! - `symbol_map_file`: TOML file mapping `raw_symbol → canonical_symbol`
//! - `channel_map`: inline descriptor table mapping `raw_channel → canonical_channel`
//!
//! Behavior:
//! - `normalize_symbol(raw)` → mapped value if present, else `raw` (passthrough)
//! - `normalize_channel(raw)` → mapped value if present, else `raw` (passthrough)
//! - If map file is configured but missing/unreadable → error

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

    #[error("failed to parse symbol map TOML '{path}': {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
}

/// Loaded normalization maps.
#[derive(Debug, Clone, Default)]
pub struct NormalizationMaps {
    pub symbol_map: HashMap<String, String>,
    pub channel_map: HashMap<String, String>,
}

impl NormalizationMaps {
    /// Normalize a raw symbol. Returns the mapped value if present, else the raw value.
    pub fn normalize_symbol<'a>(&'a self, raw: &'a str) -> &'a str {
        self.symbol_map.get(raw).map(|s| s.as_str()).unwrap_or(raw)
    }

    /// Normalize a raw channel. Returns the mapped value if present, else the raw value.
    pub fn normalize_channel<'a>(&'a self, raw: &'a str) -> &'a str {
        self.channel_map.get(raw).map(|s| s.as_str()).unwrap_or(raw)
    }
}

/// Load a symbol map from a TOML file.
///
/// Expected format: a flat table of `raw_symbol = "canonical_symbol"` pairs.
///
/// ```toml
/// btcusdt = "BTC_USDT"
/// ethusdt = "ETH_USDT"
/// ```
pub fn load_symbol_map_file(path: &Path) -> Result<HashMap<String, String>, MapsError> {
    let content = std::fs::read_to_string(path).map_err(|e| MapsError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    let table: HashMap<String, String> =
        toml::from_str(&content).map_err(|e| MapsError::Parse {
            path: path.display().to_string(),
            source: e,
        })?;

    Ok(table)
}

/// Build a channel map from a descriptor's inline `channel_map` TOML table.
///
/// Each value must be a string. Non-string values are skipped with a warning
/// (in production, tracing should log this — here we silently skip).
pub fn build_channel_map(table: &toml::value::Table) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, val) in table {
        if let toml::Value::String(s) = val {
            map.insert(key.clone(), s.clone());
        }
    }
    map
}

/// Load normalization maps from descriptor configuration.
///
/// - If `symbol_map_file` is `Some`, load from the file (resolved relative to `base_dir`).
/// - If `channel_map` is `Some`, build from the inline table.
/// - Returns error if symbol map file is specified but missing/unreadable.
pub fn load_maps(
    symbol_map_file: Option<&str>,
    channel_map_table: Option<&toml::value::Table>,
    base_dir: &Path,
) -> Result<NormalizationMaps, MapsError> {
    let symbol_map = match symbol_map_file {
        Some(path_str) => {
            let p = Path::new(path_str);
            let resolved = if p.is_absolute() {
                p.to_path_buf()
            } else {
                base_dir.join(p)
            };
            load_symbol_map_file(&resolved)?
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

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn normalize_symbol_hit() {
        let mut m = NormalizationMaps::default();
        m.symbol_map
            .insert("btcusdt".to_string(), "BTC_USDT".to_string());
        assert_eq!(m.normalize_symbol("btcusdt"), "BTC_USDT");
    }

    #[test]
    fn normalize_symbol_miss_passthrough() {
        let m = NormalizationMaps::default();
        assert_eq!(m.normalize_symbol("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn normalize_channel_hit() {
        let mut m = NormalizationMaps::default();
        m.channel_map
            .insert("trade".to_string(), "trades".to_string());
        assert_eq!(m.normalize_channel("trade"), "trades");
    }

    #[test]
    fn normalize_channel_miss_passthrough() {
        let m = NormalizationMaps::default();
        assert_eq!(m.normalize_channel("orderbook"), "orderbook");
    }

    #[test]
    fn load_symbol_map_from_file() {
        let mut tmp = tempfile().unwrap();
        writeln!(tmp.0, r#"btcusdt = "BTC_USDT""#).unwrap();
        writeln!(tmp.0, r#"ethusdt = "ETH_USDT""#).unwrap();
        let map = load_symbol_map_file(Path::new(&tmp.1)).unwrap();
        assert_eq!(map.get("btcusdt").unwrap(), "BTC_USDT");
        assert_eq!(map.get("ethusdt").unwrap(), "ETH_USDT");
    }

    #[test]
    fn load_symbol_map_missing_file_errors() {
        let err = load_symbol_map_file(Path::new("/tmp/nonexistent_map_12345.toml")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("failed to read"), "got: {}", msg);
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
        assert_eq!(map.get("trade").unwrap(), "trades");
        assert_eq!(map.get("book").unwrap(), "orderbook_l2");
    }

    #[test]
    fn load_maps_with_both() {
        let mut tmp = tempfile().unwrap();
        writeln!(tmp.0, r#"btcusdt = "BTC_USDT""#).unwrap();

        let mut ch_table = toml::value::Table::new();
        ch_table.insert(
            "trade".to_string(),
            toml::Value::String("trades".to_string()),
        );

        let maps = load_maps(Some(&tmp.1), Some(&ch_table), Path::new("/")).unwrap();
        assert_eq!(maps.normalize_symbol("btcusdt"), "BTC_USDT");
        assert_eq!(maps.normalize_channel("trade"), "trades");
    }

    #[test]
    fn load_maps_no_files() {
        let maps = load_maps(None, None, Path::new("/")).unwrap();
        assert_eq!(maps.normalize_symbol("raw"), "raw");
        assert_eq!(maps.normalize_channel("raw"), "raw");
    }

    /// Helper: create a temporary file and return (file, path_string).
    fn tempfile() -> Result<(std::fs::File, String), std::io::Error> {
        let path = format!("/tmp/_crypto_collector_test_{}.toml", std::process::id());
        let file = std::fs::File::create(&path)?;
        Ok((file, path))
    }
}
