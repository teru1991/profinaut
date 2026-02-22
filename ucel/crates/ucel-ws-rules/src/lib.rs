use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Full,
    Partial,
    NotSupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementPolicy {
    PublicOnly,
    RequiresCredentials,
    OptionalCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rate {
    pub messages_per_second: Option<u32>,
    pub messages_per_hour: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeartbeatPolicy {
    pub ping_interval_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyProfile {
    pub max_streams_per_conn: usize,
    pub max_symbols_per_conn: usize,
}

impl SafetyProfile {
    pub fn conservative() -> Self {
        Self {
            max_streams_per_conn: 25,
            max_symbols_per_conn: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeWsRules {
    pub exchange_id: String,
    pub support_level: SupportLevel,
    pub rate: Option<Rate>,
    pub heartbeat: Option<HeartbeatPolicy>,
    pub entitlement: Option<EntitlementPolicy>,
    pub safety_profile: SafetyProfile,
    pub max_streams_per_conn: Option<usize>,
    pub max_symbols_per_conn: Option<usize>,
}

impl ExchangeWsRules {
    pub fn unknown(exchange_id: impl Into<String>) -> Self {
        Self {
            exchange_id: exchange_id.into(),
            support_level: SupportLevel::Unknown,
            rate: None,
            heartbeat: None,
            entitlement: None,
            safety_profile: SafetyProfile::conservative(),
            max_streams_per_conn: None,
            max_symbols_per_conn: None,
        }
    }

    pub fn effective_max_streams_per_conn(&self) -> usize {
        self.max_streams_per_conn
            .or(self.max_symbols_per_conn)
            .unwrap_or(self.safety_profile.max_streams_per_conn)
    }
}

pub fn load_rule_file(path: &Path) -> Result<ExchangeWsRules, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub fn load_rules_dir(dir: &Path) -> Result<Vec<ExchangeWsRules>, String> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) == Some("toml") {
            out.push(load_rule_file(&path)?);
        }
    }
    Ok(out)
}

pub fn load_for_exchange(dir: &Path, exchange_id: &str) -> ExchangeWsRules {
    let path: PathBuf = dir.join(format!("{exchange_id}.toml"));
    load_rule_file(&path).unwrap_or_else(|_| ExchangeWsRules::unknown(exchange_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_to_model() {
        let raw = r#"
exchange_id = "binance"
support_level = "full"
max_streams_per_conn = 100

[safety_profile]
max_streams_per_conn = 20
max_symbols_per_conn = 30

[rate]
messages_per_second = 10
messages_per_hour = 1200

[heartbeat]
ping_interval_secs = 20
idle_timeout_secs = 120

entitlement = "public_only"
"#;
        let rule: ExchangeWsRules = toml::from_str(raw).expect("toml parse");
        assert_eq!(rule.exchange_id, "binance");
        assert_eq!(rule.support_level, SupportLevel::Full);
        assert_eq!(rule.effective_max_streams_per_conn(), 100);
    }

    #[test]
    fn conservative_defaults_are_safe() {
        let unknown = ExchangeWsRules::unknown("missing");
        assert_eq!(unknown.support_level, SupportLevel::Unknown);
        assert_eq!(unknown.safety_profile.max_streams_per_conn, 25);
        assert_eq!(unknown.safety_profile.max_symbols_per_conn, 50);
    }
}
