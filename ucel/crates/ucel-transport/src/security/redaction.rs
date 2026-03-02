use serde_json::Value;

/// Redaction policy for preventing secret leakage.
/// This is deliberately conservative (mask more rather than less).
#[derive(Debug, Clone)]
pub struct RedactionPolicy {
    pub mask: String,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            mask: "***REDACTED***".to_string(),
        }
    }
}

impl RedactionPolicy {
    fn is_sensitive_key(&self, k: &str) -> bool {
        let key = k.trim().to_ascii_lowercase();
        matches!(
            key.as_str(),
            "authorization"
                | "proxy-authorization"
                | "x-api-key"
                | "api-key"
                | "apikey"
                | "secret"
                | "client-secret"
                | "signature"
                | "passphrase"
                | "cookie"
                | "set-cookie"
                | "token"
                | "access_token"
                | "refresh_token"
        ) || key.contains("secret")
            || key.contains("signature")
            || key.contains("token")
            || key.contains("password")
            || key.contains("apikey")
            || key.contains("api_key")
    }

    pub fn redact_value_if_sensitive(&self, key: &str, value: &str) -> String {
        if self.is_sensitive_key(key) {
            self.mask.clone()
        } else {
            value.to_string()
        }
    }
}

/// Redact key-value pairs (headers/query params) before logging.
pub fn redact_kv_pairs<I, K, V>(policy: &RedactionPolicy, pairs: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    pairs
        .into_iter()
        .map(|(k, v)| {
            let ks = k.into();
            let vs = v.into();
            let rv = policy.redact_value_if_sensitive(&ks, &vs);
            (ks, rv)
        })
        .collect()
}

/// Redact JSON value recursively (best-effort).
/// Rules:
/// - object keys that look sensitive are replaced with mask
/// - arrays are traversed
pub fn redact_json_value(policy: &RedactionPolicy, v: &mut Value) {
    match v {
        Value::Object(map) => {
            for (k, vv) in map.iter_mut() {
                if policy.is_sensitive_key(k) {
                    *vv = Value::String(policy.mask.clone());
                } else {
                    redact_json_value(policy, vv);
                }
            }
        }
        Value::Array(xs) => {
            for x in xs.iter_mut() {
                redact_json_value(policy, x);
            }
        }
        _ => {}
    }
}
