//! JSON Pointer extraction (RFC 6901) and typed casting utilities.

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PointerError {
    #[error("required field at pointer '{pointer}' is missing")]
    MissingRequired { pointer: String },

    #[error("value at pointer '{pointer}' has type {actual_type}, cannot cast to {target_type}")]
    CastFailed {
        pointer: String,
        actual_type: String,
        target_type: String,
    },
}

/// Extract a JSON value using an RFC 6901 JSON Pointer string.
///
/// The pointer must start with `/`. Returns `None` if the path does not exist.
pub fn extract_json_pointer<'a>(value: &'a Value, ptr: &str) -> Option<&'a Value> {
    value.pointer(ptr)
}

/// Cast a JSON value to `u64`.
///
/// Accepts JSON numbers directly and numeric strings (fully numeric only).
pub fn cast_to_u64(val: &Value) -> Option<u64> {
    match val {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

/// Cast a JSON value to `i64`.
///
/// Accepts JSON numbers directly and numeric strings (fully numeric only).
pub fn cast_to_i64(val: &Value) -> Option<i64> {
    match val {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// Cast a JSON value to `String`.
///
/// Numbers, booleans, and strings are all converted. Null returns `"null"`.
/// Objects/arrays are JSON-serialized.
pub fn cast_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Cast a JSON value to `bool`.
///
/// Accepts JSON booleans directly. Strings "true"/"false" are accepted.
pub fn cast_to_bool(val: &Value) -> Option<bool> {
    match val {
        Value::Bool(b) => Some(*b),
        Value::String(s) => match s.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Typed extraction result.
#[derive(Debug, Clone)]
pub enum TypedValue {
    U64(u64),
    I64(i64),
    Str(String),
    Bool(bool),
    Json(Value),
}

/// Cast rule for `extract_typed`.
#[derive(Debug, Clone, Copy)]
pub enum CastRule {
    U64,
    I64,
    Str,
    Bool,
    /// Return the raw JSON value without casting.
    Raw,
}

/// Extract a value from a JSON payload using a pointer, optionally cast it.
///
/// - `optional=false` + missing → error with pointer path
/// - `optional=true` + missing → `Ok(None)`
/// - Present but cast fails → error with value type and pointer
pub fn extract_typed(
    payload: &Value,
    pointer: &str,
    optional: bool,
    cast_rule: CastRule,
) -> Result<Option<TypedValue>, PointerError> {
    let val = match extract_json_pointer(payload, pointer) {
        Some(v) if !v.is_null() => v,
        _ => {
            if optional {
                return Ok(None);
            } else {
                return Err(PointerError::MissingRequired {
                    pointer: pointer.to_string(),
                });
            }
        }
    };

    let typed = match cast_rule {
        CastRule::U64 => {
            cast_to_u64(val)
                .map(TypedValue::U64)
                .ok_or_else(|| PointerError::CastFailed {
                    pointer: pointer.to_string(),
                    actual_type: json_type_name(val).to_string(),
                    target_type: "u64".to_string(),
                })?
        }
        CastRule::I64 => {
            cast_to_i64(val)
                .map(TypedValue::I64)
                .ok_or_else(|| PointerError::CastFailed {
                    pointer: pointer.to_string(),
                    actual_type: json_type_name(val).to_string(),
                    target_type: "i64".to_string(),
                })?
        }
        CastRule::Str => TypedValue::Str(cast_to_string(val)),
        CastRule::Bool => {
            cast_to_bool(val)
                .map(TypedValue::Bool)
                .ok_or_else(|| PointerError::CastFailed {
                    pointer: pointer.to_string(),
                    actual_type: json_type_name(val).to_string(),
                    target_type: "bool".to_string(),
                })?
        }
        CastRule::Raw => TypedValue::Json(val.clone()),
    };

    Ok(Some(typed))
}

fn json_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_nested_pointer() {
        let data = json!({"data": {"symbol": "BTC/USDT", "ts": 1234567890}});
        assert_eq!(
            extract_json_pointer(&data, "/data/symbol"),
            Some(&json!("BTC/USDT"))
        );
        assert_eq!(
            extract_json_pointer(&data, "/data/ts"),
            Some(&json!(1234567890))
        );
        assert!(extract_json_pointer(&data, "/data/missing").is_none());
    }

    #[test]
    fn cast_number_to_u64() {
        assert_eq!(cast_to_u64(&json!(42)), Some(42));
        assert_eq!(cast_to_u64(&json!("123")), Some(123));
        assert_eq!(cast_to_u64(&json!("abc")), None);
        assert_eq!(cast_to_u64(&json!(-1)), None);
    }

    #[test]
    fn cast_number_to_i64() {
        assert_eq!(cast_to_i64(&json!(42)), Some(42));
        assert_eq!(cast_to_i64(&json!(-5)), Some(-5));
        assert_eq!(cast_to_i64(&json!("-99")), Some(-99));
    }

    #[test]
    fn cast_to_string_variants() {
        assert_eq!(cast_to_string(&json!("hello")), "hello");
        assert_eq!(cast_to_string(&json!(42)), "42");
        assert_eq!(cast_to_string(&json!(true)), "true");
        assert_eq!(cast_to_string(&json!(null)), "null");
    }

    #[test]
    fn cast_bool() {
        assert_eq!(cast_to_bool(&json!(true)), Some(true));
        assert_eq!(cast_to_bool(&json!(false)), Some(false));
        assert_eq!(cast_to_bool(&json!("true")), Some(true));
        assert_eq!(cast_to_bool(&json!("false")), Some(false));
        assert_eq!(cast_to_bool(&json!("yes")), None);
        assert_eq!(cast_to_bool(&json!(1)), None);
    }

    #[test]
    fn extract_typed_required_missing() {
        let data = json!({"a": 1});
        let err = extract_typed(&data, "/b", false, CastRule::U64).unwrap_err();
        assert!(err.to_string().contains("/b"));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn extract_typed_optional_missing() {
        let data = json!({"a": 1});
        let result = extract_typed(&data, "/b", true, CastRule::U64).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn extract_typed_cast_failure() {
        let data = json!({"a": "not_a_number"});
        let err = extract_typed(&data, "/a", false, CastRule::U64).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("/a"), "should include pointer: {msg}");
        assert!(msg.contains("string"), "should include actual type: {msg}");
        assert!(msg.contains("u64"), "should include target type: {msg}");
    }

    #[test]
    fn extract_typed_numeric_string_casting() {
        let data = json!({"seq": "12345"});
        let result = extract_typed(&data, "/seq", false, CastRule::U64).unwrap();
        match result.unwrap() {
            TypedValue::U64(v) => assert_eq!(v, 12345),
            other => panic!("expected U64, got {other:?}"),
        }
    }

    #[test]
    fn extract_typed_raw() {
        let data = json!({"nested": {"x": 1}});
        let result = extract_typed(&data, "/nested", false, CastRule::Raw).unwrap();
        match result.unwrap() {
            TypedValue::Json(v) => assert_eq!(v, json!({"x": 1})),
            other => panic!("expected Json, got {other:?}"),
        }
    }
}
