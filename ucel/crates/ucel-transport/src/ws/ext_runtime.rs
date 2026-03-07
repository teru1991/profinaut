use serde_json::Value;
use ucel_core::{
    VendorPublicWsIntegrityMode, VendorPublicWsReadinessMode, VendorPublicWsResumeMode,
    VendorPublicWsTypedEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VendorPublicWsReadyState {
    Planned,
    Subscribed,
    Active,
    Deadletter,
}

#[derive(Debug, Clone)]
pub struct VendorPublicWsSessionConfig {
    pub readiness_mode: VendorPublicWsReadinessMode,
    pub integrity_mode: VendorPublicWsIntegrityMode,
    pub resume_mode: VendorPublicWsResumeMode,
    pub heartbeat_timeout_ms: u64,
    pub ack_timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct VendorPublicWsSubscribePlan {
    pub operation_id: String,
    pub source_channel: String,
}

pub trait VendorPublicWsSubscriber {
    fn build_subscribe_frame(&self, plan: &VendorPublicWsSubscribePlan) -> Value;
    fn build_unsubscribe_frame(&self, plan: &VendorPublicWsSubscribePlan) -> Value;
    fn handle_ack_message(&self, message: &Value) -> bool;
}

pub trait VendorPublicWsNormalizer {
    fn normalize_extension_event(&self, message: &Value) -> Option<VendorPublicWsTypedEnvelope>;
}

pub trait VendorPublicWsIntegrityAdapter {
    fn apply_snapshot(&mut self, payload: &Value);
    fn apply_delta(&mut self, payload: &Value) -> Result<(), &'static str>;
    fn verify_checksum(&self, payload: &Value) -> Result<(), &'static str>;
}

pub fn should_activate_now(
    mode: VendorPublicWsReadinessMode,
    ack_seen: bool,
    event_seen: bool,
) -> bool {
    match mode {
        VendorPublicWsReadinessMode::ExplicitAck => ack_seen,
        VendorPublicWsReadinessMode::ImplicitObservation => event_seen,
        VendorPublicWsReadinessMode::ImmediateActive => true,
    }
}

pub fn next_resume_action(mode: VendorPublicWsResumeMode) -> &'static str {
    match mode {
        VendorPublicWsResumeMode::ResubscribeOnly => "resubscribe_only",
        VendorPublicWsResumeMode::ResnapshotThenResubscribe => "resnapshot_then_resubscribe",
        VendorPublicWsResumeMode::Deadletter => "deadletter",
    }
}
