use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RedactionPolicy {
    pub placeholder: String,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            placeholder: "***redacted***".to_string(),
        }
    }
}

impl RedactionPolicy {
    pub fn is_sensitive_key(&self, key: &str) -> bool {
        let lower = key.trim().to_ascii_lowercase();
        matches!(
            lower.as_str(),
            "authorization"
                | "proxy-authorization"
                | "x-api-key"
                | "api-key"
                | "apikey"
                | "api_key"
                | "access-key"
                | "access-sign"
                | "access-signature"
                | "access-nonce"
                | "access-timestamp"
                | "signature"
                | "passphrase"
                | "session_token"
                | "session-token"
                | "token"
                | "access_token"
                | "refresh_token"
                | "secret"
                | "api_secret"
        ) || lower.contains("secret")
            || lower.contains("token")
            || lower.contains("signature")
            || lower.contains("passphrase")
    }
}

pub fn redact_headers(
    policy: &RedactionPolicy,
    headers: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    redact_map(policy, headers)
}

pub fn redact_query(
    policy: &RedactionPolicy,
    query: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    redact_map(policy, query)
}

pub fn redact_body_map(
    policy: &RedactionPolicy,
    body: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    redact_map(policy, body)
}

fn redact_map(
    policy: &RedactionPolicy,
    map: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    map.iter()
        .map(|(k, v)| {
            if policy.is_sensitive_key(k) {
                (k.clone(), policy.placeholder.clone())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

pub fn redact_sign_input_preview(policy: &RedactionPolicy, preview: &str) -> String {
    let lower = preview.to_ascii_lowercase();
    if lower.contains("api_secret")
        || lower.contains("session_token")
        || lower.contains("authorization")
        || lower.contains("signature")
        || lower.contains("passphrase")
    {
        policy.placeholder.clone()
    } else {
        preview.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_matcher_is_case_insensitive() {
        let p = RedactionPolicy::default();
        assert!(p.is_sensitive_key("Authorization"));
        assert!(p.is_sensitive_key("X-API-KEY"));
        assert!(p.is_sensitive_key("session_token"));
        assert!(!p.is_sensitive_key("symbol"));
    }
}
