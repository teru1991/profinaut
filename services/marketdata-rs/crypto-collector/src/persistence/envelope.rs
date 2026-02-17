//! Envelope v1 — canonical wrapper for raw crypto market data messages.
//!
//! This is the stable integration type for the crypto-collector pipeline.
//! All persistence operations (Mongo, spool, dedup) work with this type.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Envelope v1
// ---------------------------------------------------------------------------

/// Envelope v1: wraps a raw market data message with routing metadata.
///
/// Field priority for dedup key resolution:
/// 1. `message_id`  (exchange-provided, most specific)
/// 2. `sequence`    (monotonic sequence, scoped to exchange+channel)
/// 3. payload hash  (fallback: DefaultHasher of serialised payload)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope {
    /// Optional exchange-provided message ID (dedup priority 1).
    pub message_id: Option<String>,
    /// Optional monotonic sequence number (dedup priority 2).
    pub sequence: Option<u64>,
    /// Exchange identifier (e.g. "binance", "kraken").
    pub exchange: String,
    /// Channel type (e.g. "trades", "orderbook").
    pub channel: String,
    /// Instrument symbol (e.g. "BTC/USDT").
    pub symbol: String,
    /// Exchange server timestamp in milliseconds since epoch, if available.
    pub server_time_ms: Option<i64>,
    /// Local receive timestamp in milliseconds since epoch.
    pub received_at_ms: i64,
    /// Raw parsed payload (JSON).
    pub payload: serde_json::Value,
}

impl Envelope {
    /// Compute the dedup key for this envelope using the priority rules:
    /// 1. `message_id` → `"mid:<id>"`
    /// 2. `sequence`   → `"seq:<exchange>:<channel>:<seq>"`
    /// 3. hash         → `"hash:<16 hex chars>"`
    ///
    /// Note: The hash uses `std::hash::DefaultHasher` on the serialised payload
    /// string. JSON key ordering is not normalised, so semantically equal
    /// payloads with different key order may produce different hashes (false
    /// negatives are acceptable for dedup; false positives are not).
    pub fn dedup_key(&self) -> String {
        if let Some(ref mid) = self.message_id {
            return format!("mid:{mid}");
        }
        if let Some(seq) = self.sequence {
            return format!("seq:{}:{}:{}", self.exchange, self.channel, seq);
        }
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.payload.to_string().hash(&mut hasher);
        format!("hash:{:016x}", hasher.finish())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_env(message_id: Option<&str>, sequence: Option<u64>) -> Envelope {
        Envelope {
            message_id: message_id.map(str::to_string),
            sequence,
            exchange: "testex".to_string(),
            channel: "trades".to_string(),
            symbol: "BTC/USDT".to_string(),
            server_time_ms: None,
            received_at_ms: 1_700_000_000_000,
            payload: json!({"price": "50000", "qty": "1.0"}),
        }
    }

    #[test]
    fn dedup_key_prefers_message_id() {
        let env = make_env(Some("msg-abc"), Some(9));
        assert!(
            env.dedup_key().starts_with("mid:"),
            "expected mid: prefix, got: {}",
            env.dedup_key()
        );
        assert_eq!(env.dedup_key(), "mid:msg-abc");
    }

    #[test]
    fn dedup_key_falls_back_to_sequence() {
        let env = make_env(None, Some(42));
        assert!(
            env.dedup_key().starts_with("seq:"),
            "expected seq: prefix, got: {}",
            env.dedup_key()
        );
        assert_eq!(env.dedup_key(), "seq:testex:trades:42");
    }

    #[test]
    fn dedup_key_falls_back_to_hash() {
        let env = make_env(None, None);
        let k = env.dedup_key();
        assert!(
            k.starts_with("hash:"),
            "expected hash: prefix, got: {k}"
        );
        assert_eq!(k.len(), "hash:".len() + 16);
    }

    #[test]
    fn dedup_key_hash_stable_for_same_payload() {
        let env1 = make_env(None, None);
        let env2 = make_env(None, None);
        assert_eq!(env1.dedup_key(), env2.dedup_key());
    }

    #[test]
    fn envelope_json_roundtrip() {
        let env = Envelope {
            message_id: Some("x1".to_string()),
            sequence: Some(7),
            exchange: "binance".to_string(),
            channel: "trades".to_string(),
            symbol: "ETH/USDT".to_string(),
            server_time_ms: Some(1_700_000_000),
            received_at_ms: 1_700_000_001_000,
            payload: json!({"p": "3000", "q": "0.5"}),
        };
        let s = serde_json::to_string(&env).unwrap();
        let decoded: Envelope = serde_json::from_str(&s).unwrap();
        assert_eq!(decoded, env);
    }
}
