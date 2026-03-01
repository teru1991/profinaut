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
                if validate_stability(&r).is_ok() {
                    r
                } else {
                    conservative(exchange_id, Some(p))
                }
            }
            Err(_) => conservative(exchange_id, Some(p)),
        },
        Err(_) => conservative(exchange_id, Some(p)),
    }
}

pub fn load_for_exchange_checked(
    rules_dir: &Path,
    exchange_id: &str,
) -> Result<ExchangeWsRules, String> {
    let p = rules_dir.join(format!("{exchange_id}.toml"));
    let raw = std::fs::read_to_string(&p).map_err(|e| format!("read rules failed: {e}"))?;
    let mut r: ExchangeWsRules =
        toml::from_str(&raw).map_err(|e| format!("parse rules failed: {e}"))?;
    if r.exchange_id != exchange_id {
        r.exchange_id = exchange_id.to_string();
    }
    validate_stability(&r)?;
    Ok(r)
}

fn validate_stability(r: &crate::model::ExchangeWsRules) -> Result<(), String> {
    let Some(st) = &r.stability else {
        return Ok(());
    };

    if let Some(b) = &st.buckets {
        for (name, v) in [
            ("control_rps", b.control_rps),
            ("private_rps", b.private_rps),
            ("public_rps", b.public_rps),
        ] {
            if let Some(x) = v {
                if x <= 0.0 || !x.is_finite() {
                    return Err(format!("stability.buckets.{name} must be finite and > 0"));
                }
            }
        }
    }

    if let Some(rl) = &st.rate_limit {
        if let Some(m) = rl.max_attempts {
            if m <= 0 {
                return Err("stability.rate_limit.max_attempts must be > 0".into());
            }
        }
        if let Some(b) = rl.base_cooldown_secs {
            if b <= 0 {
                return Err("stability.rate_limit.base_cooldown_secs must be > 0".into());
            }
        }
        if let Some(m) = rl.max_cooldown_secs {
            if m <= 0 {
                return Err("stability.rate_limit.max_cooldown_secs must be > 0".into());
            }
        }
    }

    if let Some(cb) = &st.circuit_breaker {
        if let Some(x) = cb.failure_threshold {
            if x == 0 {
                return Err("stability.circuit_breaker.failure_threshold must be >= 1".into());
            }
        }
        if let Some(x) = cb.success_threshold {
            if x == 0 {
                return Err("stability.circuit_breaker.success_threshold must be >= 1".into());
            }
        }
        if let Some(x) = cb.cooldown_ms {
            if x == 0 {
                return Err("stability.circuit_breaker.cooldown_ms must be >= 1".into());
            }
        }
        if let Some(x) = cb.half_open_max_trials {
            if x == 0 {
                return Err("stability.circuit_breaker.half_open_max_trials must be >= 1".into());
            }
        }
    }

    if let Some(of) = &st.overflow {
        if let Some(mode) = &of.mode {
            let m = mode.to_ascii_lowercase();
            let ok = matches!(
                m.as_str(),
                "drop_newest"
                    | "drop_oldest_low_priority"
                    | "slowdown_then_drop_oldest_low_priority"
                    | "spill_to_disk_then_drop_oldest_low_priority"
            );
            if !ok {
                return Err("stability.overflow.mode is unknown".into());
            }
            if m == "spill_to_disk_then_drop_oldest_low_priority"
                && of
                    .spill_dir
                    .as_ref()
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
            {
                return Err("stability.overflow.spill_dir is required when spill_to_disk_*".into());
            }
        }
    }

    Ok(())
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
        stability: None,
        max_streams_per_conn: None,
        max_symbols_per_conn: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_loader_rejects_invalid_stability() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("x.toml");
        std::fs::write(
            &p,
            r#"
exchange_id = "x"
support_level = "full"

[stability.buckets]
public_rps = 0
"#,
        )
        .unwrap();

        let e = load_for_exchange_checked(d.path(), "x").unwrap_err();
        assert!(e.contains("stability.buckets.public_rps"));
    }

    #[test]
    fn checked_loader_accepts_valid_stability() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("x.toml");
        std::fs::write(
            &p,
            r#"
exchange_id = "x"
support_level = "full"

[stability.overflow]
mode = "spill_to_disk_then_drop_oldest_low_priority"
spill_dir = "/tmp/spool"
"#,
        )
        .unwrap();

        let r = load_for_exchange_checked(d.path(), "x").unwrap();
        assert!(r.stability.is_some());
    }
}
