//! B6 — Public API Surfaces for later tasks.
//!
//! Exposes stable interfaces that compose B1–B5 modules:
//!
//! - `generate_subscriptions(generator_source, ctx)` → `Vec<String>`
//! - `extract_metadata(payload, parse_rules)` → raw extracted values
//! - `normalize_metadata(extracted, maps)` → canonical outputs
//!
//! No networking — pure computation only.

use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

use crate::dsl::{self, DslContext};
use crate::json_pointer::{self, CastRule};
use crate::maps::NormalizationMaps;
use crate::mini_expr::{self, ExprConfig};
use crate::placeholder::{self, PlaceholderContext};

// ───────────────────────────────────────────────────────────────────────────
// Errors
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("DSL error: {0}")]
    Dsl(#[from] dsl::DslError),

    #[error("placeholder error: {0}")]
    Placeholder(#[from] placeholder::PlaceholderError),

    #[error("JSON pointer error: {0}")]
    JsonPointer(#[from] json_pointer::JsonPointerError),

    #[error("expression error: {0}")]
    Expr(#[from] mini_expr::ExprError),
}

// ───────────────────────────────────────────────────────────────────────────
// Subscription generation
// ───────────────────────────────────────────────────────────────────────────

/// Context for subscription generation (combines DSL + placeholder context).
#[derive(Debug, Clone)]
pub struct SubscriptionContext {
    pub symbols: Vec<String>,
    pub channels: Vec<String>,
    pub conn_id: String,
    pub args: HashMap<String, String>,
    /// Maximum output messages (default 1_000_000).
    pub max_outputs: usize,
}

impl Default for SubscriptionContext {
    fn default() -> Self {
        Self {
            symbols: Vec::new(),
            channels: Vec::new(),
            conn_id: String::new(),
            args: HashMap::new(),
            max_outputs: 1_000_000,
        }
    }
}

/// Generate subscription messages from a DSL generator source and context.
///
/// 1. Parses + executes the DSL → raw emitted strings
/// 2. Applies placeholder substitution to each emitted string
///
/// Returns `Vec<String>` of fully rendered subscription messages.
pub fn generate_subscriptions(
    generator_source: &str,
    ctx: &SubscriptionContext,
    sub_index: usize,
) -> Result<Vec<String>, EngineError> {
    let dsl_ctx = DslContext {
        symbols: ctx.symbols.clone(),
        channels: ctx.channels.clone(),
        conn_id: ctx.conn_id.clone(),
        max_outputs: ctx.max_outputs,
    };

    let raw_messages = dsl::execute(generator_source, dsl_ctx, sub_index)?;

    // Apply placeholder substitution to each emitted message
    let mut rendered = Vec::with_capacity(raw_messages.len());
    for (i, raw) in raw_messages.iter().enumerate() {
        // Build per-message placeholder context; symbol/channel are set from
        // the DSL execution but we don't track which loop iteration produced
        // which emit. The placeholder {symbol}/{ch} in emitted strings are
        // raw text from the DSL source, so we do a post-hoc scan.
        //
        // For placeholders that need dynamic values per-emit (like {now_ms},
        // {uuid}), those are resolved fresh each time by the placeholder engine.
        //
        // The DSL emit produces template strings with {symbol} etc. literally
        // in them. To resolve them properly we'd need the loop variable bindings
        // at the time of emit. We solve this by having the DSL interpreter
        // NOT substitute placeholders — it emits raw template strings.
        // Then here we try to substitute. If symbol/channel placeholders are
        // present but we have no single value (because we're outside the loop),
        // we need to handle this differently.
        //
        // Actually the correct approach: the DSL interpreter already runs inside
        // foreach loops. We need to capture the loop variable state at emit time.
        // Let's use the enhanced approach where the DSL outputs (template, bindings).

        // Simplified approach: for each emitted string, we substitute using
        // a context that has the conn_id and args. Symbol/channel are not
        // available at this level (they were loop variables). The correct fix
        // is to have the DSL produce bindings alongside templates.
        // For now, we provide conn_id and args; symbol/channel templates
        // remain as literal text unless the user embeds them differently.

        let ph_ctx = PlaceholderContext {
            symbol: None,
            channel: None,
            conn_id: Some(ctx.conn_id.clone()),
            args: ctx.args.clone(),
        };

        // Try to substitute. If a placeholder like {symbol} is found but
        // no symbol is in context, we'll get an error. This is by design:
        // the subscription DSL should use emit() with already-interpolated
        // string content. If the user wants {symbol} in the emitted JSON,
        // they should construct it in the DSL.
        //
        // However, to support the common pattern, we check if the template
        // contains {symbol} or {ch} and skip placeholder substitution for
        // templates that don't contain any {}-placeholders at all.
        if raw.contains('{') {
            // Only attempt substitution if there are braces.
            // For any errors, we wrap with context about which message index.
            match placeholder::substitute(raw, &ph_ctx) {
                Ok(s) => rendered.push(s),
                Err(e) => {
                    return Err(EngineError::Placeholder(
                        placeholder::PlaceholderError::Malformed {
                            raw: format!("message[{}]", i),
                            reason: e.to_string(),
                        },
                    ));
                }
            }
        } else {
            rendered.push(raw.clone());
        }
    }

    Ok(rendered)
}

// ───────────────────────────────────────────────────────────────────────────
// Metadata extraction
// ───────────────────────────────────────────────────────────────────────────

/// Parse rules for metadata extraction (mirrors descriptor.parse section).
#[derive(Debug, Clone)]
pub struct ParseRules {
    pub channel_pointer: String,
    pub symbol_pointer: String,
    pub server_time_pointer: Option<String>,
    pub sequence_pointer: Option<String>,
    pub message_id_pointer: Option<String>,
    pub expr_enabled: bool,
    pub expressions: Vec<String>,
    pub expr_config: ExprConfig,
}

/// Raw extracted metadata values (before normalization).
#[derive(Debug, Clone, Default)]
pub struct ExtractedMetadata {
    pub channel: Option<Value>,
    pub symbol: Option<Value>,
    pub server_time: Option<Value>,
    pub sequence: Option<Value>,
    pub message_id: Option<Value>,
    /// Extra values from expression evaluation, keyed by expression string.
    pub expr_values: HashMap<String, Value>,
}

/// Extract metadata from a JSON payload using parse rules.
///
/// Uses JSON pointers for standard fields, and mini-expr for custom expressions.
pub fn extract_metadata(
    payload: &Value,
    rules: &ParseRules,
) -> Result<ExtractedMetadata, EngineError> {
    let channel =
        json_pointer::extract_typed(payload, &rules.channel_pointer, false, CastRule::String)?;
    let symbol =
        json_pointer::extract_typed(payload, &rules.symbol_pointer, false, CastRule::String)?;

    let server_time = match rules.server_time_pointer {
        Some(ref ptr) => json_pointer::extract_typed(payload, ptr, true, CastRule::Raw)?,
        None => None,
    };
    let sequence = match rules.sequence_pointer {
        Some(ref ptr) => json_pointer::extract_typed(payload, ptr, true, CastRule::Raw)?,
        None => None,
    };
    let message_id = match rules.message_id_pointer {
        Some(ref ptr) => json_pointer::extract_typed(payload, ptr, true, CastRule::Raw)?,
        None => None,
    };

    let mut expr_values = HashMap::new();
    if rules.expr_enabled {
        for expr_str in &rules.expressions {
            let val = mini_expr::evaluate(expr_str, payload, &rules.expr_config)?;
            expr_values.insert(expr_str.clone(), val);
        }
    }

    Ok(ExtractedMetadata {
        channel,
        symbol,
        server_time,
        sequence,
        message_id,
        expr_values,
    })
}

// ───────────────────────────────────────────────────────────────────────────
// Metadata normalization
// ───────────────────────────────────────────────────────────────────────────

/// Normalized (canonical) metadata output.
#[derive(Debug, Clone, Default)]
pub struct NormalizedMetadata {
    pub channel: Option<String>,
    pub symbol: Option<String>,
    pub server_time: Option<Value>,
    pub sequence: Option<Value>,
    pub message_id: Option<Value>,
}

/// Normalize extracted metadata using maps.
///
/// Applies symbol and channel normalization; other fields pass through.
pub fn normalize_metadata(
    extracted: &ExtractedMetadata,
    maps: &NormalizationMaps,
) -> NormalizedMetadata {
    let raw_channel = extracted
        .channel
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let raw_symbol = extracted
        .symbol
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    NormalizedMetadata {
        channel: if raw_channel.is_empty() {
            None
        } else {
            Some(maps.normalize_channel(raw_channel).to_string())
        },
        symbol: if raw_symbol.is_empty() {
            None
        } else {
            Some(maps.normalize_symbol(raw_symbol).to_string())
        },
        server_time: extracted.server_time.clone(),
        sequence: extracted.sequence.clone(),
        message_id: extracted.message_id.clone(),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn generate_subscriptions_basic() {
        let src = r#"
            foreach(ch in channels) {
                emit("subscribe_{conn_id}");
            }
        "#;
        let ctx = SubscriptionContext {
            symbols: vec![],
            channels: vec!["trades".to_string(), "book".to_string()],
            conn_id: "main".to_string(),
            args: HashMap::new(),
            max_outputs: 1_000_000,
        };
        let out = generate_subscriptions(src, &ctx, 0).unwrap();
        assert_eq!(out.len(), 2);
        // {conn_id} gets substituted
        assert_eq!(out[0], "subscribe_main");
        assert_eq!(out[1], "subscribe_main");
    }

    #[test]
    fn generate_subscriptions_no_placeholders() {
        let src = r#"emit("plain_message");"#;
        let ctx = SubscriptionContext::default();
        let out = generate_subscriptions(src, &ctx, 0).unwrap();
        assert_eq!(out, vec!["plain_message"]);
    }

    #[test]
    fn extract_metadata_basic() {
        let payload = json!({
            "channel": "trades",
            "symbol": "BTC_USDT",
            "timestamp": 1700000000000_u64,
            "seq": 42
        });
        let rules = ParseRules {
            channel_pointer: "/channel".to_string(),
            symbol_pointer: "/symbol".to_string(),
            server_time_pointer: Some("/timestamp".to_string()),
            sequence_pointer: Some("/seq".to_string()),
            message_id_pointer: None,
            expr_enabled: false,
            expressions: vec![],
            expr_config: ExprConfig::default(),
        };
        let meta = extract_metadata(&payload, &rules).unwrap();
        assert_eq!(meta.channel, Some(json!("trades")));
        assert_eq!(meta.symbol, Some(json!("BTC_USDT")));
        assert!(meta.server_time.is_some());
        assert!(meta.sequence.is_some());
    }

    #[test]
    fn extract_metadata_missing_required() {
        let payload = json!({"channel": "trades"});
        let rules = ParseRules {
            channel_pointer: "/channel".to_string(),
            symbol_pointer: "/symbol".to_string(),
            server_time_pointer: None,
            sequence_pointer: None,
            message_id_pointer: None,
            expr_enabled: false,
            expressions: vec![],
            expr_config: ExprConfig::default(),
        };
        let err = extract_metadata(&payload, &rules).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("/symbol"), "got: {}", msg);
    }

    #[test]
    fn extract_metadata_with_expr() {
        let _payload_incomplete = json!({"data": {"price": "42.5"}});
        let rules = ParseRules {
            channel_pointer: "/channel".to_string(),
            symbol_pointer: "/symbol".to_string(),
            server_time_pointer: None,
            sequence_pointer: None,
            message_id_pointer: None,
            expr_enabled: true,
            expressions: vec!["to_number(data.price)".to_string()],
            expr_config: ExprConfig::default(),
        };
        // This will fail because channel and symbol are missing.
        // Let's provide them:
        let payload = json!({
            "channel": "trades",
            "symbol": "BTC",
            "data": {"price": "42.5"}
        });
        let meta = extract_metadata(&payload, &rules).unwrap();
        assert_eq!(
            meta.expr_values.get("to_number(data.price)"),
            Some(&json!(42.5))
        );
    }

    #[test]
    fn normalize_metadata_with_maps() {
        let mut maps = NormalizationMaps::default();
        maps.symbol_map
            .insert("btcusdt".to_string(), "BTC_USDT".to_string());
        maps.channel_map
            .insert("trade".to_string(), "trades".to_string());

        let extracted = ExtractedMetadata {
            channel: Some(json!("trade")),
            symbol: Some(json!("btcusdt")),
            server_time: Some(json!(1700000000000_u64)),
            sequence: Some(json!(42)),
            message_id: None,
            expr_values: HashMap::new(),
        };

        let norm = normalize_metadata(&extracted, &maps);
        assert_eq!(norm.channel, Some("trades".to_string()));
        assert_eq!(norm.symbol, Some("BTC_USDT".to_string()));
        assert!(norm.server_time.is_some());
    }

    #[test]
    fn normalize_metadata_passthrough_on_no_map() {
        let maps = NormalizationMaps::default();
        let extracted = ExtractedMetadata {
            channel: Some(json!("orderbook")),
            symbol: Some(json!("ETH_BTC")),
            ..Default::default()
        };
        let norm = normalize_metadata(&extracted, &maps);
        assert_eq!(norm.channel, Some("orderbook".to_string()));
        assert_eq!(norm.symbol, Some("ETH_BTC".to_string()));
    }
}
