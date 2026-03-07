use crate::diagnostics::scan::{ResidualFinding, ResidualScanner};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedactionRules {
    pub version: String,
    pub deny_header_keys: Vec<String>,
    pub deny_json_keys: Vec<String>,
    pub deny_value_patterns: Vec<String>,
    pub replacement: String,
}

impl Default for RedactionRules {
    fn default() -> Self {
        Self {
            version: "1".into(),
            deny_header_keys: vec![
                "authorization".into(),
                "cookie".into(),
                "set-cookie".into(),
                "x-api-key".into(),
                "x-signature".into(),
            ],
            deny_json_keys: vec![
                "password".into(),
                "secret".into(),
                "token".into(),
                "api_key".into(),
                "apikey".into(),
                "access_key".into(),
                "private_key".into(),
                "signature".into(),
                "nonce".into(),
            ],
            deny_value_patterns: vec![
                r"(?i)authorization\s*:".into(),
                r"(?i)cookie\s*:".into(),
                r"[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}".into(),
                r"-----BEGIN [A-Z ]+PRIVATE KEY-----".into(),
                r"-----BEGIN [A-Z ]+KEY-----".into(),
                r"(?i)\bbearer\s+[A-Za-z0-9._\-+/=]{10,}\b".into(),
            ],
            replacement: "[REDACTED]".into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedactionError {
    #[error("redaction rules invalid: {0}")]
    InvalidRules(String),
    #[error("regex compile error: {0}")]
    Regex(String),
    #[error("residual secrets detected after redaction (fail-closed)")]
    ResidualDetected { findings: Vec<ResidualFinding> },
}

pub struct Redactor {
    deny_json_keys_lc: Vec<String>,
    value_res: Vec<Regex>,
    scanner: ResidualScanner,
    replacement: String,
}

impl Redactor {
    pub fn new(rules: &RedactionRules) -> Result<Self, RedactionError> {
        if rules.replacement.is_empty() {
            return Err(RedactionError::InvalidRules("replacement empty".into()));
        }

        let mut value_res = Vec::with_capacity(rules.deny_value_patterns.len());
        for p in &rules.deny_value_patterns {
            value_res.push(Regex::new(p).map_err(|e| RedactionError::Regex(e.to_string()))?);
        }

        let scanner = ResidualScanner::new(&rules.deny_value_patterns)
            .map_err(|e| RedactionError::Regex(format!("{e}")))?;

        Ok(Self {
            deny_json_keys_lc: rules
                .deny_json_keys
                .iter()
                .map(|s| s.to_ascii_lowercase())
                .collect(),
            value_res,
            scanner,
            replacement: rules.replacement.clone(),
        })
    }

    pub fn redact_json_value(&self, v: &mut Value) {
        match v {
            Value::Object(map) => {
                for (k, vv) in map.iter_mut() {
                    if self.deny_json_keys_lc.contains(&k.to_ascii_lowercase()) {
                        *vv = Value::String(self.replacement.clone());
                        continue;
                    }
                    self.redact_json_value(vv);
                }
            }
            Value::Array(xs) => {
                for x in xs {
                    self.redact_json_value(x);
                }
            }
            Value::String(s) => {
                let mut out = s.clone();
                for re in &self.value_res {
                    out = re.replace_all(&out, self.replacement.as_str()).to_string();
                }
                *s = out;
            }
            _ => {}
        }
    }

    pub fn redact_text(&self, s: &str) -> String {
        let mut out = s.to_string();
        for re in &self.value_res {
            out = re.replace_all(&out, self.replacement.as_str()).to_string();
        }
        out
    }

    pub fn has_deny_pattern(&self, bytes: &[u8]) -> bool {
        !self.scanner.scan(bytes).is_empty()
    }

    pub fn fail_closed_scan(&self, bytes: &[u8]) -> Result<(), RedactionError> {
        let findings = self.scanner.scan(bytes);
        if findings.is_empty() {
            Ok(())
        } else {
            Err(RedactionError::ResidualDetected { findings })
        }
    }
}
