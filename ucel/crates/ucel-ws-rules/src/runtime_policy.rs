use crate::model::ExchangeWsRules;
use ucel_core::{IngestHeartbeatPolicy, IngestIntegrityMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePolicy {
    pub heartbeat: IngestHeartbeatPolicy,
    pub integrity: IngestIntegrityMode,
    pub expects_ack: bool,
    pub requires_resume_snapshot: bool,
}

pub fn runtime_policy_for(rules: &ExchangeWsRules, is_private: bool) -> RuntimePolicy {
    let hb = rules.heartbeat.clone();
    RuntimePolicy {
        heartbeat: IngestHeartbeatPolicy {
            ping_interval_ms: hb.as_ref().and_then(|h| h.ping_interval_secs).unwrap_or(10) * 1000,
            idle_timeout_ms: hb.as_ref().and_then(|h| h.idle_timeout_secs).unwrap_or(30) * 1000,
            stall_after_ms: hb.as_ref().and_then(|h| h.idle_timeout_secs).unwrap_or(30) * 1000,
        },
        integrity: if is_private {
            IngestIntegrityMode::None
        } else {
            IngestIntegrityMode::Sequence
        },
        expects_ack: is_private,
        requires_resume_snapshot: !is_private,
    }
}
