//! B3 — JSON Pointer Extraction + Casting Utilities (RFC 6901).
//!
//! # Extraction
//! - `extract_json_pointer(value, ptr)` — navigate a `serde_json::Value` using RFC 6901 pointer
//!
//! # Casting
//! - `cast_to_u64`, `cast_to_i64`, `cast_to_string`, `cast_to_bool`
//! - Numeric strings: accepted if fully numeric, else error (documented policy)
//!
//! # Typed wrapper
//! - `extract_typed(payload, pointer, optional, cast_rule)` — combines extraction + casting

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonPointerError {
    #[error("missing required value at pointer '{pointer}'")]
    MissingRequired { pointer: String },

    #[error("cast failed at pointer '{pointer}': cannot convert {actual_type} to {target_type}")]
    CastFailed {
        pointer: String,
        actual_type: String,
        target_type: String,
    },

    #[error("invalid pointer syntax: '{pointer}'")]
    InvalidPointer { pointer: String },
}

/// The cast target types supported by `extract_typed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastRule {
    U64,
    I64,
    String,
    Bool,
    /// Return the raw `Value` without casting.
    Raw,
}

// ───────────────────────────────────────────────────────────────────────────
// RFC 6901 pointer resolution
// ───────────────────────────────────────────────────────────────────────────

/// Extract a value from `root` using an RFC 6901 JSON Pointer string.
///
/// Returns `None` if the path does not exist.
/// The pointer must start with `/` (the empty string `""` refers to the root).
pub fn extract_json_pointer<'a>(root: &'a Value, ptr: &str) -> Option<&'a Value> {
    if ptr.is_empty() {
        return Some(root);
    }
    if !ptr.starts_with('/') {
        return None;
    }

    let mut current = root;
    for segment in ptr[1..].split('/') {
        // RFC 6901 escape: ~1 → '/', ~0 → '~'
        let unescaped = segment.replace("~1", "/").replace("~0", "~");

        match current {
            Value::Object(map) => {
                current = map.get(&unescaped)?;
            }
            Value::Array(arr) => {
                let idx: usize = unescaped.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

// ───────────────────────────────────────────────────────────────────────────
// Casting utilities
// ───────────────────────────────────────────────────────────────────────────

/// Cast a JSON value to `u64`.
///
/// Policy for numeric strings: accepted if the entire string is a valid non-negative integer.
pub fn cast_to_u64(value: &Value) -> Result<u64, String> {
    match value {
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(u)
            } else if let Some(f) = n.as_f64() {
                if f >= 0.0 && f <= u64::MAX as f64 && f.fract() == 0.0 {
                    Ok(f as u64)
                } else {
                    Err(format!("number {} cannot be represented as u64", n))
                }
            } else {
                Err(format!("number {} cannot be represented as u64", n))
            }
        }
        Value::String(s) => s
            .parse::<u64>()
            .map_err(|_| format!("string '{}' is not a valid u64", s)),
        _ => Err(format!("cannot cast {} to u64", value_type_name(value))),
    }
}

/// Cast a JSON value to `i64`.
///
/// Policy for numeric strings: accepted if the entire string is a valid integer.
pub fn cast_to_i64(value: &Value) -> Result<i64, String> {
    match value {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i)
            } else if let Some(f) = n.as_f64() {
                if f >= i64::MIN as f64 && f <= i64::MAX as f64 && f.fract() == 0.0 {
                    Ok(f as i64)
                } else {
                    Err(format!("number {} cannot be represented as i64", n))
                }
            } else {
                Err(format!("number {} cannot be represented as i64", n))
            }
        }
        Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| format!("string '{}' is not a valid i64", s)),
        _ => Err(format!("cannot cast {} to i64", value_type_name(value))),
    }
}

/// Cast a JSON value to `String`.
pub fn cast_to_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => Err(format!("cannot cast {} to string", value_type_name(value))),
    }
}

/// Cast a JSON value to `bool`.
pub fn cast_to_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        Value::String(s) => match s.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(format!("string '{}' is not a valid bool", s)),
        },
        _ => Err(format!("cannot cast {} to bool", value_type_name(value))),
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Typed extractor
// ───────────────────────────────────────────────────────────────────────────

/// Extract a value at `pointer` from `payload`, optionally cast it.
///
/// - If `optional` is `false` and the pointer is missing → `Err`
/// - If `optional` is `true` and the pointer is missing → `Ok(None)`
/// - If present but cast fails → `Err` (includes value type and pointer)
pub fn extract_typed(
    payload: &Value,
    pointer: &str,
    optional: bool,
    cast_rule: CastRule,
) -> Result<Option<Value>, JsonPointerError> {
    let extracted = extract_json_pointer(payload, pointer);

    match extracted {
        None | Some(Value::Null) => {
            if optional {
                Ok(None)
            } else {
                Err(JsonPointerError::MissingRequired {
                    pointer: pointer.to_string(),
                })
            }
        }
        Some(val) => {
            let casted =
                match cast_rule {
                    CastRule::U64 => cast_to_u64(val)
                        .map(|v| Value::Number(serde_json::Number::from(v)))
                        .map_err(|_| JsonPointerError::CastFailed {
                            pointer: pointer.to_string(),
                            actual_type: value_type_name(val).to_string(),
                            target_type: "u64".to_string(),
                        }),
                    CastRule::I64 => cast_to_i64(val)
                        .map(|v| Value::Number(serde_json::Number::from(v)))
                        .map_err(|_| JsonPointerError::CastFailed {
                            pointer: pointer.to_string(),
                            actual_type: value_type_name(val).to_string(),
                            target_type: "i64".to_string(),
                        }),
                    CastRule::String => cast_to_string(val).map(Value::String).map_err(|_| {
                        JsonPointerError::CastFailed {
                            pointer: pointer.to_string(),
                            actual_type: value_type_name(val).to_string(),
                            target_type: "string".to_string(),
                        }
                    }),
                    CastRule::Bool => cast_to_bool(val).map(Value::Bool).map_err(|_| {
                        JsonPointerError::CastFailed {
                            pointer: pointer.to_string(),
                            actual_type: value_type_name(val).to_string(),
                            target_type: "bool".to_string(),
                        }
                    }),
                    CastRule::Raw => Ok(val.clone()),
                }?;
            Ok(Some(casted))
        }
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
    fn extract_nested_object() {
        let v = json!({"a": {"b": {"c": 42}}});
        let r = extract_json_pointer(&v, "/a/b/c").unwrap();
        assert_eq!(r, &json!(42));
    }

    #[test]
    fn extract_array_index() {
        let v = json!({"items": [10, 20, 30]});
        let r = extract_json_pointer(&v, "/items/1").unwrap();
        assert_eq!(r, &json!(20));
    }

    #[test]
    fn extract_missing_returns_none() {
        let v = json!({"a": 1});
        assert!(extract_json_pointer(&v, "/b").is_none());
    }

    #[test]
    fn extract_rfc6901_escapes() {
        let v = json!({"a/b": {"~c": 99}});
        let r = extract_json_pointer(&v, "/a~1b/~0c").unwrap();
        assert_eq!(r, &json!(99));
    }

    #[test]
    fn extract_empty_pointer_returns_root() {
        let v = json!({"x": 1});
        let r = extract_json_pointer(&v, "").unwrap();
        assert_eq!(r, &v);
    }

    #[test]
    fn cast_u64_from_number() {
        assert_eq!(cast_to_u64(&json!(42)).unwrap(), 42);
    }

    #[test]
    fn cast_u64_from_numeric_string() {
        assert_eq!(cast_to_u64(&json!("123")).unwrap(), 123);
    }

    #[test]
    fn cast_u64_from_non_numeric_string_fails() {
        assert!(cast_to_u64(&json!("abc")).is_err());
    }

    #[test]
    fn cast_i64_negative() {
        assert_eq!(cast_to_i64(&json!(-5)).unwrap(), -5);
    }

    #[test]
    fn cast_string_from_number() {
        assert_eq!(cast_to_string(&json!(42)).unwrap(), "42");
    }

    #[test]
    fn cast_bool_from_true() {
        assert!(cast_to_bool(&json!(true)).unwrap());
    }

    #[test]
    fn cast_bool_from_string() {
        assert!(cast_to_bool(&json!("true")).unwrap());
        assert!(!cast_to_bool(&json!("false")).unwrap());
    }

    #[test]
    fn cast_bool_from_invalid_string_fails() {
        assert!(cast_to_bool(&json!("yes")).is_err());
    }

    #[test]
    fn extract_typed_required_missing_errors() {
        let v = json!({"a": 1});
        let err = extract_typed(&v, "/b", false, CastRule::U64).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing required"), "got: {}", msg);
        assert!(msg.contains("/b"), "got: {}", msg);
    }

    #[test]
    fn extract_typed_optional_missing_returns_none() {
        let v = json!({"a": 1});
        let r = extract_typed(&v, "/b", true, CastRule::U64).unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn extract_typed_cast_fail_includes_pointer() {
        let v = json!({"x": "not_a_number"});
        let err = extract_typed(&v, "/x", false, CastRule::U64).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("/x"), "got: {}", msg);
        assert!(msg.contains("string"), "got: {}", msg);
        assert!(msg.contains("u64"), "got: {}", msg);
    }

    #[test]
    fn extract_typed_success_u64() {
        let v = json!({"seq": 42});
        let r = extract_typed(&v, "/seq", false, CastRule::U64)
            .unwrap()
            .unwrap();
        assert_eq!(r, json!(42));
    }

    #[test]
    fn extract_typed_null_treated_as_missing() {
        let v = json!({"x": null});
        let r = extract_typed(&v, "/x", true, CastRule::String).unwrap();
        assert!(r.is_none());

        let err = extract_typed(&v, "/x", false, CastRule::String).unwrap_err();
        assert!(err.to_string().contains("missing required"));
    }
}
