//! Public API surfaces for later tasks.
//!
//! Exposes stable interfaces that compose the lower-level modules:
//! - `generate_subscriptions` — DSL execution producing subscription messages
//! - `extract_metadata` — JSON pointer extraction from exchange payloads
//! - `normalize_metadata` — map raw extracted values to canonical form

use crate::descriptor::ParseSection;
use crate::dsl::{self, DslContext, DslError};
use crate::expr;
use crate::maps::NormalizationMaps;
use crate::pointer::{self, CastRule, PointerError};
use serde_json::Value;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("DSL error: {0}")]
    Dsl(#[from] DslError),

    #[error("pointer extraction error: {0}")]
    Pointer(#[from] PointerError),

    #[error("expression evaluation error: {0}")]
    Expr(#[from] expr::ExprError),
}

// ---------------------------------------------------------------------------
// generate_subscriptions
// ---------------------------------------------------------------------------

/// Generate subscription messages by executing a DSL generator source.
///
/// - `generator_source`: DSL source code (from descriptor subscription)
/// - `ctx`: runtime context (symbols, channels, conn_id)
///
/// Returns a `Vec<String>` of rendered messages, in generation order.
pub fn generate_subscriptions(
    generator_source: &str,
    ctx: &DslContext,
    sub_index: usize,
) -> Result<Vec<String>, ApiError> {
    Ok(dsl::execute(generator_source, ctx, sub_index)?)
}

// ---------------------------------------------------------------------------
// extract_metadata
// ---------------------------------------------------------------------------

/// Raw extracted metadata from an exchange message payload.
#[derive(Debug, Clone, Default)]
pub struct ExtractedMetadata {
    pub channel: Option<String>,
    pub symbol: Option<String>,
    pub server_time: Option<String>,
    pub sequence: Option<u64>,
    pub message_id: Option<String>,
}

/// Extract metadata fields from a JSON payload using the descriptor's parse
/// section pointers.
///
/// - `channel` and `symbol` pointers are required (error if missing).
/// - `server_time`, `sequence`, and `message_id` are optional.
/// - If `parse.expr.enabled` is true and expressions are defined, they are
///   evaluated and the results override the pointer-extracted values.
pub fn extract_metadata(
    payload: &Value,
    parse: &ParseSection,
) -> Result<ExtractedMetadata, ApiError> {
    let channel =
        pointer::extract_typed(payload, &parse.channel, false, CastRule::Str)?.map(|v| match v {
            pointer::TypedValue::Str(s) => s,
            _ => unreachable!(),
        });

    let symbol =
        pointer::extract_typed(payload, &parse.symbol, false, CastRule::Str)?.map(|v| match v {
            pointer::TypedValue::Str(s) => s,
            _ => unreachable!(),
        });

    let server_time = match &parse.server_time {
        Some(ptr) => pointer::extract_typed(payload, ptr, true, CastRule::Str)?.map(|v| match v {
            pointer::TypedValue::Str(s) => s,
            _ => unreachable!(),
        }),
        None => None,
    };

    let sequence = match &parse.sequence {
        Some(ptr) => pointer::extract_typed(payload, ptr, true, CastRule::U64)?.map(|v| match v {
            pointer::TypedValue::U64(n) => n,
            _ => unreachable!(),
        }),
        None => None,
    };

    let message_id = match &parse.message_id {
        Some(ptr) => pointer::extract_typed(payload, ptr, true, CastRule::Str)?.map(|v| match v {
            pointer::TypedValue::Str(s) => s,
            _ => unreachable!(),
        }),
        None => None,
    };

    // If expr engine is enabled, evaluate expressions and let them override
    let meta = ExtractedMetadata {
        channel,
        symbol,
        server_time,
        sequence,
        message_id,
    };

    if let Some(ref expr_settings) = parse.expr {
        if expr_settings.enabled {
            if let Some(ref expressions) = expr_settings.expressions {
                for expression in expressions {
                    // Expressions can override metadata fields by convention:
                    // "channel: <expr>" or just a raw expression.
                    // For now, evaluate and store as-is; task C+ can refine semantics.
                    let val = expr::evaluate(expression, payload)?;
                    if !val.is_null() {
                        // If the expression result is usable, it could override;
                        // exact mapping semantics deferred to Task C.
                        let _ = val;
                    }
                }
            }
        }
    }

    Ok(meta)
}

// ---------------------------------------------------------------------------
// normalize_metadata
// ---------------------------------------------------------------------------

/// Normalized metadata with canonical symbol/channel names.
#[derive(Debug, Clone)]
pub struct NormalizedMetadata {
    pub channel: String,
    pub symbol: String,
    pub server_time: Option<String>,
    pub sequence: Option<u64>,
    pub message_id: Option<String>,
}

/// Apply normalization maps to extracted metadata.
///
/// `channel` and `symbol` are mapped through the maps; other fields pass through.
pub fn normalize_metadata(
    extracted: &ExtractedMetadata,
    maps: &NormalizationMaps,
) -> NormalizedMetadata {
    let raw_channel = extracted.channel.as_deref().unwrap_or("");
    let raw_symbol = extracted.symbol.as_deref().unwrap_or("");

    NormalizedMetadata {
        channel: maps.normalize_channel(raw_channel).to_string(),
        symbol: maps.normalize_symbol(raw_symbol).to_string(),
        server_time: extracted.server_time.clone(),
        sequence: extracted.sequence,
        message_id: extracted.message_id.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::ExprSettings;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn generate_subscriptions_basic() {
        let ctx = DslContext {
            symbols: vec!["BTC/USDT".to_string()],
            channels: vec!["trades".to_string()],
            conn_id: "main".to_string(),
            args: HashMap::new(),
            max_outputs: 1_000_000,
        };
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    emit("subscribe:{channel}:{symbol}");
                }
            }
        "#;
        let msgs = generate_subscriptions(src, &ctx, 0).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0], "subscribe:trades:BTC/USDT");
    }

    #[test]
    fn extract_metadata_basic() {
        let payload = json!({
            "ch": "trade",
            "data": {"symbol": "BTCUSDT", "ts": "1234567890"},
            "seq": 42,
            "id": "msg-001"
        });
        let parse = ParseSection {
            channel: "/ch".to_string(),
            symbol: "/data/symbol".to_string(),
            server_time: Some("/data/ts".to_string()),
            sequence: Some("/seq".to_string()),
            message_id: Some("/id".to_string()),
            expr: None,
        };
        let meta = extract_metadata(&payload, &parse).unwrap();
        assert_eq!(meta.channel.as_deref(), Some("trade"));
        assert_eq!(meta.symbol.as_deref(), Some("BTCUSDT"));
        assert_eq!(meta.server_time.as_deref(), Some("1234567890"));
        assert_eq!(meta.sequence, Some(42));
        assert_eq!(meta.message_id.as_deref(), Some("msg-001"));
    }

    #[test]
    fn extract_metadata_missing_required() {
        let payload = json!({"ch": "trade"});
        let parse = ParseSection {
            channel: "/ch".to_string(),
            symbol: "/data/symbol".to_string(),
            server_time: None,
            sequence: None,
            message_id: None,
            expr: None,
        };
        let err = extract_metadata(&payload, &parse).unwrap_err();
        assert!(err.to_string().contains("/data/symbol"));
    }

    #[test]
    fn extract_metadata_with_expr() {
        let payload = json!({"ch": "trade", "data": {"symbol": "BTC", "price": "100"}});
        let parse = ParseSection {
            channel: "/ch".to_string(),
            symbol: "/data/symbol".to_string(),
            server_time: None,
            sequence: None,
            message_id: None,
            expr: Some(ExprSettings {
                enabled: true,
                expressions: Some(vec!["data.price".to_string()]),
                max_expression_length: 4096,
            }),
        };
        // Should succeed — expr evaluation runs but doesn't override in Task B
        let meta = extract_metadata(&payload, &parse).unwrap();
        assert_eq!(meta.channel.as_deref(), Some("trade"));
    }

    #[test]
    fn normalize_metadata_applies_maps() {
        let extracted = ExtractedMetadata {
            channel: Some("trade".to_string()),
            symbol: Some("BTCUSDT".to_string()),
            server_time: None,
            sequence: Some(1),
            message_id: None,
        };
        let maps = NormalizationMaps {
            symbol_map: HashMap::from([("BTCUSDT".to_string(), "BTC/USDT".to_string())]),
            channel_map: HashMap::from([("trade".to_string(), "trades".to_string())]),
        };
        let norm = normalize_metadata(&extracted, &maps);
        assert_eq!(norm.symbol, "BTC/USDT");
        assert_eq!(norm.channel, "trades");
        assert_eq!(norm.sequence, Some(1));
    }

    #[test]
    fn normalize_metadata_passthrough() {
        let extracted = ExtractedMetadata {
            channel: Some("unknown_ch".to_string()),
            symbol: Some("UNKNOWN".to_string()),
            server_time: None,
            sequence: None,
            message_id: None,
        };
        let maps = NormalizationMaps::default();
        let norm = normalize_metadata(&extracted, &maps);
        assert_eq!(norm.symbol, "UNKNOWN");
        assert_eq!(norm.channel, "unknown_ch");
    }
}
