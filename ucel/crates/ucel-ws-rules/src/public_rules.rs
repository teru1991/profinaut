use crate::model::{ExchangeWsRules, SupportLevel};
use ucel_core::{PublicWsAckMode, PublicWsIntegrityMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicRuleView {
    pub ack_mode: PublicWsAckMode,
    pub integrity_mode: PublicWsIntegrityMode,
    pub supports_resume: bool,
    pub supports_heartbeat: bool,
}

pub fn public_rule_view(rules: &ExchangeWsRules) -> PublicRuleView {
    let supports_heartbeat = rules.heartbeat.is_some();
    let public_rps = rules
        .stability
        .as_ref()
        .and_then(|s| s.buckets.as_ref())
        .and_then(|b| b.public_rps)
        .unwrap_or_default();

    PublicRuleView {
        ack_mode: PublicWsAckMode::ImplicitObservation,
        integrity_mode: if public_rps > 0.0 {
            PublicWsIntegrityMode::SequenceOnly
        } else {
            PublicWsIntegrityMode::None
        },
        supports_resume: !matches!(rules.support_level, SupportLevel::NotSupported),
        supports_heartbeat,
    }
}
