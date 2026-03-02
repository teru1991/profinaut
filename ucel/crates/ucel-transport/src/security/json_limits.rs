use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone, Copy)]
pub struct JsonLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
}

impl Default for JsonLimits {
    fn default() -> Self {
        Self {
            max_bytes: 512 * 1024,
            max_depth: 64,
        }
    }
}

/// Check JSON limits without parsing full JSON.
/// - Enforces size
/// - Enforces nesting depth of { } and [ ]
/// - Best-effort string handling (skips braces inside strings)
pub fn check_json_limits(input: &[u8], limits: JsonLimits) -> Result<(), UcelError> {
    if input.len() > limits.max_bytes {
        return Err(UcelError::new(
            ErrorCode::WsProtocolViolation,
            format!(
                "json too large: {} bytes (max {})",
                input.len(),
                limits.max_bytes
            ),
        ));
    }

    let mut depth: usize = 0;
    let mut in_str = false;
    let mut escape = false;

    for &b in input {
        if in_str {
            if escape {
                escape = false;
                continue;
            }
            if b == b'\\' {
                escape = true;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            continue;
        } else if b == b'"' {
            in_str = true;
            continue;
        }

        match b {
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                if depth > limits.max_depth {
                    return Err(UcelError::new(
                        ErrorCode::WsProtocolViolation,
                        format!("json depth exceeded: {depth} (max {})", limits.max_depth),
                    ));
                }
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    Ok(())
}
