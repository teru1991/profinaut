use std::path::{Path, PathBuf};

use crate::model::{ExchangeWsRules, HeartbeatPolicy, RatePolicy, SafetyProfile, SupportLevel};

pub fn load_for_exchange(rules_dir: &Path, exchange_id: &str) -> ExchangeWsRules {
    let p = rules_dir.join(format!("{exchange_id}.toml"));
    match std::fs::read_to_string(&p) {
        Ok(raw) => match toml::from_str::<ExchangeWsRules>(&raw) {
            Ok(mut r) => {
                if r.exchange_id != exchange_id {
                    r.exchange_id = exchange_id.to_string();
                }
                r
            }
            Err(_) => conservative(exchange_id, Some(p)),
        },
        Err(_) => conservative(exchange_id, Some(p)),
    }
}

fn conservative(exchange_id: &str, _path: Option<PathBuf>) -> ExchangeWsRules {
    ExchangeWsRules {
        exchange_id: exchange_id.to_string(),
        support_level: SupportLevel::Partial,
        rate: Some(RatePolicy {
            messages_per_second: Some(1),
            messages_per_hour: Some(3600),
        }),
        heartbeat: Some(HeartbeatPolicy {
            ping_interval_secs: Some(20),
            idle_timeout_secs: Some(30),
            max_connection_age_secs: Some(0),
        }),
        entitlement: Some("public_only".to_string()),
        safety_profile: Some(SafetyProfile {
            max_streams_per_conn: Some(25),
            max_symbols_per_conn: Some(50),
        }),
        max_streams_per_conn: None,
        max_symbols_per_conn: None,
    }
}
