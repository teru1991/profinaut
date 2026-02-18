//! B2 — Placeholder Substitution Engine.
//!
//! Performs plain string substitution on emitted message templates.
//!
//! # Supported placeholders
//! - `{symbol}` — current symbol value
//! - `{ch}` — current channel value
//! - `{channel}` — alias for `{ch}`
//! - `{conn_id}` — generator-bound connection ID
//! - `{now_ms}` — Unix epoch milliseconds at evaluation time (per message)
//! - `{uuid}` — UUID v4 generated per occurrence
//! - `{env:VAR}` — environment variable `VAR`
//! - `{arg:KEY}` — argument from supplied context map
//!
//! # Policy
//! - Unknown placeholder → error (names the unknown placeholder)
//! - Missing env var → error (do not silently produce invalid messages)
//! - No nested templates, no formatting directives, no evaluation

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlaceholderError {
    #[error("unknown placeholder '{{{name}}}'")]
    Unknown { name: String },

    #[error("missing environment variable '{name}' in placeholder '{{env:{name}}}'")]
    MissingEnvVar { name: String },

    #[error("missing argument '{key}' in placeholder '{{arg:{key}}}'")]
    MissingArg { key: String },

    #[error("malformed placeholder '{raw}': {reason}")]
    Malformed { raw: String, reason: String },
}

/// Runtime context for placeholder substitution.
#[derive(Debug, Clone, Default)]
pub struct PlaceholderContext {
    pub symbol: Option<String>,
    pub channel: Option<String>,
    pub conn_id: Option<String>,
    pub args: HashMap<String, String>,
}

/// Check if a character is valid inside a placeholder name.
/// Allows: alphanumeric, underscore, colon (for env:/arg: prefixes).
fn is_placeholder_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':'
}

/// Try to extract a placeholder name starting at `start` in `chars`.
/// Returns `Some((name, closing_brace_index))` if a valid placeholder `{name}` is found,
/// where `name` consists only of valid placeholder characters and ends with `}`.
/// Returns `None` if the content doesn't look like a placeholder (treats `{` as literal).
fn try_extract_placeholder(chars: &[char], start: usize) -> Option<(String, usize)> {
    let len = chars.len();
    let mut i = start;

    // Scan placeholder name characters
    while i < len && is_placeholder_char(chars[i]) {
        i += 1;
    }

    // Must have consumed at least one character and hit a closing brace
    if i > start && i < len && chars[i] == '}' {
        let name: String = chars[start..i].iter().collect();
        Some((name, i))
    } else {
        None
    }
}

/// Substitute all `{…}` placeholders in `template`, returning the rendered string.
///
/// Only `{name}` patterns where `name` consists of alphanumeric, underscore, and colon
/// characters are treated as placeholders. All other uses of `{` are treated as literals,
/// allowing JSON content to coexist with placeholders in templates.
pub fn substitute(template: &str, ctx: &PlaceholderContext) -> Result<String, PlaceholderError> {
    let mut result = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            if let Some((name, end)) = try_extract_placeholder(&chars, i + 1) {
                let value = resolve_placeholder(&name, ctx)?;
                result.push_str(&value);
                i = end + 1; // skip past closing '}'
            } else {
                // Not a valid placeholder pattern; treat '{' as literal
                result.push('{');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Ok(result)
}

fn resolve_placeholder(name: &str, ctx: &PlaceholderContext) -> Result<String, PlaceholderError> {
    // env:VAR
    if let Some(var_name) = name.strip_prefix("env:") {
        return std::env::var(var_name).map_err(|_| PlaceholderError::MissingEnvVar {
            name: var_name.to_string(),
        });
    }

    // arg:KEY
    if let Some(key) = name.strip_prefix("arg:") {
        return ctx
            .args
            .get(key)
            .cloned()
            .ok_or_else(|| PlaceholderError::MissingArg {
                key: key.to_string(),
            });
    }

    match name {
        "symbol" => ctx.symbol.clone().ok_or_else(|| PlaceholderError::Unknown {
            name: name.to_string(),
        }),
        "ch" | "channel" => ctx
            .channel
            .clone()
            .ok_or_else(|| PlaceholderError::Unknown {
                name: name.to_string(),
            }),
        "conn_id" => ctx
            .conn_id
            .clone()
            .ok_or_else(|| PlaceholderError::Unknown {
                name: name.to_string(),
            }),
        "now_ms" => {
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            Ok(ms.to_string())
        }
        "uuid" => Ok(uuid::Uuid::new_v4().to_string()),
        _ => Err(PlaceholderError::Unknown {
            name: name.to_string(),
        }),
    }
}

/// Validate that all placeholders in `template` are recognized.
///
/// Returns `Ok(set_of_placeholder_names)` if all known, or `Err` listing unknowns.
pub fn validate_placeholders(template: &str) -> Result<Vec<String>, PlaceholderError> {
    let mut found = Vec::new();
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            if let Some((name, end)) = try_extract_placeholder(&chars, i + 1) {
                // Check if known
                let known = matches!(
                    name.as_str(),
                    "symbol" | "ch" | "channel" | "conn_id" | "now_ms" | "uuid"
                ) || name.starts_with("env:")
                    || name.starts_with("arg:");

                if !known {
                    return Err(PlaceholderError::Unknown { name });
                }

                found.push(name);
                i = end + 1;
            } else {
                // Not a placeholder pattern; skip literal '{'
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(found)
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> PlaceholderContext {
        PlaceholderContext {
            symbol: Some("BTC_USDT".to_string()),
            channel: Some("trades".to_string()),
            conn_id: Some("main".to_string()),
            args: HashMap::new(),
        }
    }

    #[test]
    fn substitute_basic_placeholders() {
        let ctx = test_ctx();
        let out = substitute("{symbol}|{ch}|{channel}|{conn_id}", &ctx).unwrap();
        assert_eq!(out, "BTC_USDT|trades|trades|main");
    }

    #[test]
    fn substitute_now_ms() {
        let ctx = test_ctx();
        let out = substitute("{now_ms}", &ctx).unwrap();
        let ms: u64 = out.parse().expect("should be numeric");
        assert!(ms > 1_000_000_000_000); // after year 2001
    }

    #[test]
    fn substitute_uuid_format() {
        let ctx = test_ctx();
        let out = substitute("{uuid}", &ctx).unwrap();
        // UUID v4: 8-4-4-4-12 hex
        assert_eq!(out.len(), 36);
        assert_eq!(out.chars().filter(|c| *c == '-').count(), 4);
        // Each occurrence should be unique
        let out2 = substitute("{uuid}", &ctx).unwrap();
        assert_ne!(out, out2);
    }

    #[test]
    fn substitute_env_var_success() {
        std::env::set_var("_TEST_PH_VAR", "secret123");
        let ctx = test_ctx();
        let out = substitute("{env:_TEST_PH_VAR}", &ctx).unwrap();
        assert_eq!(out, "secret123");
        std::env::remove_var("_TEST_PH_VAR");
    }

    #[test]
    fn substitute_env_var_missing() {
        std::env::remove_var("_NONEXISTENT_VAR_12345");
        let ctx = test_ctx();
        let err = substitute("{env:_NONEXISTENT_VAR_12345}", &ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing environment variable"), "got: {}", msg);
        assert!(msg.contains("_NONEXISTENT_VAR_12345"), "got: {}", msg);
    }

    #[test]
    fn substitute_arg_success() {
        let mut ctx = test_ctx();
        ctx.args.insert("api_key".to_string(), "key123".to_string());
        let out = substitute("{arg:api_key}", &ctx).unwrap();
        assert_eq!(out, "key123");
    }

    #[test]
    fn substitute_arg_missing() {
        let ctx = test_ctx();
        let err = substitute("{arg:missing_key}", &ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing argument"), "got: {}", msg);
        assert!(msg.contains("missing_key"), "got: {}", msg);
    }

    #[test]
    fn substitute_unknown_placeholder() {
        let ctx = test_ctx();
        let err = substitute("{bogus}", &ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown placeholder"), "got: {}", msg);
        assert!(msg.contains("bogus"), "got: {}", msg);
    }

    #[test]
    fn validate_placeholders_known() {
        let found =
            validate_placeholders("{symbol}_{ch}_{conn_id}_{now_ms}_{uuid}_{env:X}_{arg:Y}")
                .unwrap();
        assert_eq!(found.len(), 7);
    }

    #[test]
    fn validate_placeholders_unknown() {
        let err = validate_placeholders("{symbol}_{bad}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown placeholder"), "got: {}", msg);
    }

    #[test]
    fn no_placeholders_passthrough() {
        let ctx = test_ctx();
        let out = substitute("plain text no braces", &ctx).unwrap();
        assert_eq!(out, "plain text no braces");
    }

    #[test]
    fn mixed_text_and_placeholders() {
        let ctx = test_ctx();
        let out = substitute(
            r#"{"method":"subscribe","pair":"{symbol}","channel":"{ch}"}"#,
            &ctx,
        )
        .unwrap();
        assert_eq!(
            out,
            r#"{"method":"subscribe","pair":"BTC_USDT","channel":"trades"}"#
        );
    }
}
