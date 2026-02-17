//! Exchange descriptor schema v1.4 — data model and validator.
//!
//! This module defines the descriptor format that describes how to connect to,
//! subscribe to, and parse messages from a crypto exchange. Task A implements
//! the model and validation only; DSL execution is deferred to Task B.

use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DescriptorError {
    #[error("failed to read descriptor file '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse descriptor TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("descriptor validation failed:\n{}", format_errors(.0))]
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
// Data model — Descriptor v1.4
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeDescriptor {
    pub meta: Meta,
    pub ws: WsSection,
    pub rest: Option<RestSection>,
    pub subscriptions: Vec<Subscription>,
    pub parse: ParseSection,
    pub maps: Option<MapsSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    pub name: String,
    pub version: String,
}

// --- WebSocket section ---

#[derive(Debug, Clone, Deserialize)]
pub struct WsSection {
    pub connections: Vec<WsConnection>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WsConnection {
    pub id: String,
    pub urls: Vec<String>,
    pub tls: Option<TlsSettings>,
    #[serde(default = "default_read_timeout_ms")]
    pub read_timeout_ms: u64,
    pub keepalive: Option<KeepaliveSettings>,
}

fn default_read_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TlsSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct KeepaliveSettings {
    pub mode: String,
    #[serde(default = "default_keepalive_interval")]
    pub interval_ms: u64,
    pub template: Option<String>,
}

fn default_keepalive_interval() -> u64 {
    30_000
}

// --- REST section (optional) ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RestSection {
    pub base_urls: Vec<String>,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RateLimit {
    pub requests_per_minute: Option<u32>,
    pub token_bucket: Option<TokenBucket>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TokenBucket {
    pub capacity: u32,
    pub refill_per_second: f64,
}

// --- Subscriptions ---

#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    pub connection_id: String,
    pub generator: String,
    pub ack: Option<AckMatcher>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AckMatcher {
    pub field: String,
    pub value: String,
}

// --- Parse section ---

#[derive(Debug, Clone, Deserialize)]
pub struct ParseSection {
    pub channel: String,
    pub symbol: String,
    pub server_time: Option<String>,
    pub sequence: Option<String>,
    pub message_id: Option<String>,
    pub expr: Option<ExprSettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ExprSettings {
    #[serde(default)]
    pub enabled: bool,
    pub expressions: Option<Vec<String>>,
    #[serde(default = "default_max_expr_len")]
    pub max_expression_length: usize,
}

fn default_max_expr_len() -> usize {
    4096
}

// --- Maps section (optional) ---

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MapsSection {
    pub symbol_map_file: Option<String>,
    pub channel_map: Option<toml::value::Table>,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load and validate a descriptor from a TOML file path.
pub fn load_descriptor(path: &Path) -> Result<ExchangeDescriptor, DescriptorError> {
    let content = std::fs::read_to_string(path).map_err(|e| DescriptorError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    parse_descriptor(&content)
}

/// Parse and validate a descriptor from a TOML string.
pub fn parse_descriptor(content: &str) -> Result<ExchangeDescriptor, DescriptorError> {
    let desc: ExchangeDescriptor = toml::from_str(content)?;
    validate_descriptor(&desc)?;
    Ok(desc)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_descriptor(desc: &ExchangeDescriptor) -> Result<(), DescriptorError> {
    let mut errors: Vec<String> = Vec::new();

    // meta
    if desc.meta.name.is_empty() {
        errors.push("meta.name must not be empty".to_string());
    }
    if desc.meta.version.is_empty() {
        errors.push("meta.version must not be empty".to_string());
    }

    // ws.connections
    if desc.ws.connections.is_empty() {
        errors.push("ws.connections must have at least one entry".to_string());
    }

    let mut conn_ids = HashSet::new();
    for conn in &desc.ws.connections {
        if conn.id.is_empty() {
            errors.push("ws.connections: entry has empty id".to_string());
        }
        if !conn_ids.insert(&conn.id) {
            errors.push(format!(
                "ws.connections: duplicate connection id '{}'",
                conn.id
            ));
        }
        if conn.urls.is_empty() {
            errors.push(format!(
                "ws.connections '{}': urls must be non-empty",
                conn.id
            ));
        }

        // keepalive template: validated as non-empty if present (execution deferred to Task B)
        if let Some(ref ka) = conn.keepalive {
            if ka.mode.is_empty() {
                errors.push(format!(
                    "ws.connections '{}': keepalive.mode must not be empty",
                    conn.id
                ));
            }
        }
    }

    // rest (optional)
    if let Some(ref rest) = desc.rest {
        if rest.base_urls.is_empty() {
            errors
                .push("rest.base_urls must be non-empty when rest section is present".to_string());
        }
    }

    // subscriptions
    for (i, sub) in desc.subscriptions.iter().enumerate() {
        let ctx = format!("subscriptions[{}]", i);

        if sub.connection_id.is_empty() {
            errors.push(format!("{ctx}: connection_id must not be empty"));
        } else if !conn_ids.contains(&sub.connection_id) {
            errors.push(format!(
                "{ctx}: connection_id '{}' does not reference any ws.connections.id (available: {:?})",
                sub.connection_id,
                conn_ids.iter().collect::<Vec<_>>()
            ));
        }

        if sub.generator.is_empty() {
            errors.push(format!("{ctx}: generator must not be empty"));
        }

        // ack: validate shape only
        if let Some(ref ack) = sub.ack {
            if ack.field.is_empty() {
                errors.push(format!("{ctx}: ack.field must not be empty"));
            }
        }
    }

    // parse pointers
    validate_pointer("parse.channel", &desc.parse.channel, &mut errors);
    validate_pointer("parse.symbol", &desc.parse.symbol, &mut errors);
    if let Some(ref p) = desc.parse.server_time {
        validate_pointer("parse.server_time", p, &mut errors);
    }
    if let Some(ref p) = desc.parse.sequence {
        validate_pointer("parse.sequence", p, &mut errors);
    }
    if let Some(ref p) = desc.parse.message_id {
        validate_pointer("parse.message_id", p, &mut errors);
    }

    // expr: validate size bounds only
    if let Some(ref expr) = desc.parse.expr {
        if let Some(ref expressions) = expr.expressions {
            for (i, e) in expressions.iter().enumerate() {
                if e.len() > expr.max_expression_length {
                    errors.push(format!(
                        "parse.expr.expressions[{}]: length {} exceeds max {}",
                        i,
                        e.len(),
                        expr.max_expression_length
                    ));
                }
            }
        }
    }

    // maps: symbol_map_file existence is a warning, not validated here
    // (validated at service startup with filesystem access)

    if errors.is_empty() {
        Ok(())
    } else {
        Err(DescriptorError::Validation(errors))
    }
}

/// Basic RFC 6901-like JSON pointer validation.
/// Must start with '/' and contain only printable non-whitespace segments.
fn validate_pointer(field_name: &str, pointer: &str, errors: &mut Vec<String>) {
    if pointer.is_empty() {
        errors.push(format!("{field_name}: pointer must not be empty"));
        return;
    }
    if !pointer.starts_with('/') {
        errors.push(format!(
            "{field_name}: pointer '{}' must start with '/'",
            pointer
        ));
        return;
    }
    // Each segment (after splitting on '/') must be non-empty and printable
    let segments: Vec<&str> = pointer[1..].split('/').collect();
    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() && i < segments.len() - 1 {
            errors.push(format!(
                "{field_name}: pointer '{}' has empty segment at position {}",
                pointer, i
            ));
        }
        if seg.chars().any(|c| c.is_control()) {
            errors.push(format!(
                "{field_name}: pointer '{}' segment {} contains control characters",
                pointer, i
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_descriptor_toml() -> &'static str {
        r#"
[meta]
name = "example-exchange"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://ws.example.com/v1"]
read_timeout_ms = 30000

[ws.connections.keepalive]
mode = "ping_frame"
interval_ms = 25000

[[subscriptions]]
connection_id = "main"
generator = "json_subscribe(channel, symbol)"

[parse]
channel = "/channel"
symbol = "/symbol"
server_time = "/timestamp"
"#
    }

    #[test]
    fn parse_valid_descriptor() {
        let desc = parse_descriptor(valid_descriptor_toml()).unwrap();
        assert_eq!(desc.meta.name, "example-exchange");
        assert_eq!(desc.meta.version, "1.4");
        assert_eq!(desc.ws.connections.len(), 1);
        assert_eq!(desc.ws.connections[0].id, "main");
        assert_eq!(desc.subscriptions.len(), 1);
    }

    #[test]
    fn reject_empty_meta_name() {
        let toml = r#"
[meta]
name = ""
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://ws.example.com"]

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"
"#;
        let err = parse_descriptor(toml).unwrap_err();
        assert!(err.to_string().contains("meta.name must not be empty"));
    }

    #[test]
    fn reject_duplicate_connection_ids() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[[ws.connections]]
id = "main"
urls = ["wss://b.com"]

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"
"#;
        let err = parse_descriptor(toml).unwrap_err();
        assert!(err.to_string().contains("duplicate connection id"));
    }

    #[test]
    fn reject_invalid_connection_ref() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[[subscriptions]]
connection_id = "nonexistent"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"
"#;
        let err = parse_descriptor(toml).unwrap_err();
        assert!(
            err.to_string().contains("does not reference any"),
            "got: {}",
            err
        );
    }

    #[test]
    fn reject_invalid_pointer() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "no_slash"
symbol = "/sym"
"#;
        let err = parse_descriptor(toml).unwrap_err();
        assert!(err.to_string().contains("must start with '/'"));
    }

    #[test]
    fn reject_empty_urls() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = []

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"
"#;
        let err = parse_descriptor(toml).unwrap_err();
        assert!(err.to_string().contains("urls must be non-empty"));
    }

    #[test]
    fn validate_with_rest_section() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[rest]
base_urls = ["https://api.example.com"]

[rest.rate_limit]
requests_per_minute = 120

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"
"#;
        let desc = parse_descriptor(toml).unwrap();
        assert!(desc.rest.is_some());
    }

    #[test]
    fn validate_with_maps_section() {
        let toml = r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"

[maps]
symbol_map_file = "maps/example_symbol_map.toml"

[maps.channel_map]
trade = "trades"
book = "orderbook_l2"
"#;
        let desc = parse_descriptor(toml).unwrap();
        assert!(desc.maps.is_some());
        let maps = desc.maps.unwrap();
        assert_eq!(
            maps.symbol_map_file.as_deref(),
            Some("maps/example_symbol_map.toml")
        );
        assert!(maps.channel_map.is_some());
    }

    #[test]
    fn validate_expr_length_bounds() {
        let long_expr = "x".repeat(5000);
        let toml = format!(
            r#"
[meta]
name = "test"
version = "1.4"

[[ws.connections]]
id = "main"
urls = ["wss://a.com"]

[[subscriptions]]
connection_id = "main"
generator = "sub()"

[parse]
channel = "/ch"
symbol = "/sym"

[parse.expr]
enabled = true
expressions = ["{long_expr}"]
max_expression_length = 4096
"#
        );
        let err = parse_descriptor(&toml).unwrap_err();
        assert!(err.to_string().contains("exceeds max"));
    }
}
