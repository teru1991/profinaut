//! Placeholder substitution engine for emitted DSL strings.
//!
//! Supported placeholders:
//! - `{symbol}` — current symbol value
//! - `{ch}` / `{channel}` — current channel value
//! - `{conn_id}` — generator-bound connection ID
//! - `{now_ms}` — unix epoch milliseconds at evaluation time
//! - `{uuid}` — UUID v4 per occurrence
//! - `{env:VAR}` — environment variable
//! - `{arg:KEY}` — context argument (must be provided)
//!
//! Policy: unknown placeholders and missing env vars produce errors.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlaceholderError {
    #[error("unknown placeholder '{{{name}}}'")]
    Unknown { name: String },

    #[error("missing environment variable '{var}' in placeholder '{{env:{var}}}'")]
    MissingEnv { var: String },

    #[error("missing argument '{key}' in placeholder '{{arg:{key}}}'")]
    MissingArg { key: String },

    #[error("unclosed placeholder starting at byte offset {offset}")]
    Unclosed { offset: usize },
}

/// Runtime context for placeholder substitution.
#[derive(Debug, Clone, Default)]
pub struct PlaceholderContext {
    pub symbol: Option<String>,
    pub channel: Option<String>,
    pub conn_id: Option<String>,
    pub args: HashMap<String, String>,
}

/// Substitute all `{...}` placeholders in `template` using the given `ctx`.
///
/// Each `{uuid}` occurrence generates a fresh UUID v4.
/// Each `{now_ms}` occurrence captures the current time.
pub fn substitute(template: &str, ctx: &PlaceholderContext) -> Result<String, PlaceholderError> {
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            // Escaped literal brace: {{ => {
            if i + 1 < len && bytes[i + 1] == b'{' {
                result.push('{');
                i += 2;
                continue;
            }
            let start = i;
            i += 1;
            // Find closing '}'
            let mut end = None;
            while i < len {
                if bytes[i] == b'}' {
                    end = Some(i);
                    break;
                }
                i += 1;
            }
            let end = end.ok_or(PlaceholderError::Unclosed { offset: start })?;
            let name = &template[start + 1..end];
            let replacement = resolve_placeholder(name, ctx)?;
            result.push_str(&replacement);
            i = end + 1;
        } else if bytes[i] == b'}' && i + 1 < len && bytes[i + 1] == b'}' {
            // Escaped literal brace: }} => }
            result.push('}');
            i += 2;
        } else {
            result.push(template[i..].chars().next().unwrap());
            i += template[i..].chars().next().unwrap().len_utf8();
        }
    }

    Ok(result)
}

fn resolve_placeholder(name: &str, ctx: &PlaceholderContext) -> Result<String, PlaceholderError> {
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
            let ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            Ok(ms.to_string())
        }
        "uuid" => Ok(uuid::Uuid::new_v4().to_string()),
        _ => {
            if let Some(var) = name.strip_prefix("env:") {
                std::env::var(var).map_err(|_| PlaceholderError::MissingEnv {
                    var: var.to_string(),
                })
            } else if let Some(key) = name.strip_prefix("arg:") {
                ctx.args
                    .get(key)
                    .cloned()
                    .ok_or_else(|| PlaceholderError::MissingArg {
                        key: key.to_string(),
                    })
            } else {
                Err(PlaceholderError::Unknown {
                    name: name.to_string(),
                })
            }
        }
    }
}

/// Validate placeholders in a template string without substituting.
///
/// Returns the set of placeholder names found, or an error if any are malformed.
pub fn validate_placeholders(template: &str) -> Result<Vec<String>, PlaceholderError> {
    let mut names = Vec::new();
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            // Skip escaped literal braces
            if i + 1 < len && bytes[i + 1] == b'{' {
                i += 2;
                continue;
            }
            let start = i;
            i += 1;
            let mut end = None;
            while i < len {
                if bytes[i] == b'}' {
                    end = Some(i);
                    break;
                }
                i += 1;
            }
            let end = end.ok_or(PlaceholderError::Unclosed { offset: start })?;
            let name = &template[start + 1..end];

            // Check if the placeholder name is recognized
            let known = matches!(
                name,
                "symbol" | "ch" | "channel" | "conn_id" | "now_ms" | "uuid"
            ) || name.starts_with("env:")
                || name.starts_with("arg:");

            if !known {
                return Err(PlaceholderError::Unknown {
                    name: name.to_string(),
                });
            }

            names.push(name.to_string());
            i = end + 1;
        } else {
            i += 1;
        }
    }

    Ok(names)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> PlaceholderContext {
        PlaceholderContext {
            symbol: Some("BTC/USDT".to_string()),
            channel: Some("trades".to_string()),
            conn_id: Some("main".to_string()),
            args: HashMap::from([("api_key".to_string(), "secret123".to_string())]),
        }
    }

    #[test]
    fn basic_substitution() {
        let result =
            substitute(r#"{{"channel":"{channel}","symbol":"{symbol}"}}"#, &ctx()).unwrap();
        assert_eq!(result, r#"{"channel":"trades","symbol":"BTC/USDT"}"#);
    }

    #[test]
    fn escaped_braces() {
        let result = substitute("{{literal}}", &ctx()).unwrap();
        assert_eq!(result, "{literal}");
    }

    #[test]
    fn ch_alias() {
        let result = substitute("{ch}", &ctx()).unwrap();
        assert_eq!(result, "trades");
    }

    #[test]
    fn conn_id_substitution() {
        let result = substitute("conn={conn_id}", &ctx()).unwrap();
        assert_eq!(result, "conn=main");
    }

    #[test]
    fn now_ms_produces_numeric() {
        let result = substitute("{now_ms}", &ctx()).unwrap();
        assert!(result.parse::<u128>().is_ok(), "not numeric: {result}");
    }

    #[test]
    fn uuid_produces_valid_format() {
        let result = substitute("{uuid}", &ctx()).unwrap();
        assert_eq!(result.len(), 36, "UUID should be 36 chars: {result}");
        // UUID v4 format: 8-4-4-4-12
        let parts: Vec<&str> = result.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn uuid_unique_per_occurrence() {
        let result = substitute("{uuid},{uuid}", &ctx()).unwrap();
        let parts: Vec<&str> = result.split(',').collect();
        assert_ne!(parts[0], parts[1]);
    }

    #[test]
    fn env_substitution_success() {
        std::env::set_var("TEST_PLACEHOLDER_VAR", "hello_env");
        let result = substitute("{env:TEST_PLACEHOLDER_VAR}", &ctx()).unwrap();
        assert_eq!(result, "hello_env");
        std::env::remove_var("TEST_PLACEHOLDER_VAR");
    }

    #[test]
    fn env_missing_errors() {
        let err = substitute("{env:DEFINITELY_NOT_SET_XYZ123}", &ctx()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("DEFINITELY_NOT_SET_XYZ123"), "got: {msg}");
        assert!(msg.contains("missing environment variable"), "got: {msg}");
    }

    #[test]
    fn arg_substitution() {
        let result = substitute("{arg:api_key}", &ctx()).unwrap();
        assert_eq!(result, "secret123");
    }

    #[test]
    fn arg_missing_errors() {
        let err = substitute("{arg:nonexistent}", &ctx()).unwrap_err();
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn unknown_placeholder_errors() {
        let err = substitute("{banana}", &ctx()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("banana"), "got: {msg}");
        assert!(msg.contains("unknown"), "got: {msg}");
    }

    #[test]
    fn unclosed_placeholder_errors() {
        let err = substitute("hello {world", &ctx()).unwrap_err();
        assert!(err.to_string().contains("unclosed"));
    }

    #[test]
    fn validate_known_placeholders() {
        let names =
            validate_placeholders("{symbol}_{ch}_{conn_id}_{now_ms}_{uuid}_{env:X}_{arg:Y}")
                .unwrap();
        assert_eq!(names.len(), 7);
    }

    #[test]
    fn validate_catches_unknown() {
        let err = validate_placeholders("{symbol}_{bad}").unwrap_err();
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn no_placeholders_passthrough() {
        let result = substitute("plain text", &ctx()).unwrap();
        assert_eq!(result, "plain text");
    }
}
