use ucel_core::{
    CanonicalPrivateWsEvent, PrivateWsAckMode, PrivateWsChannel, PrivateWsLifecycleState,
    PrivateWsRejectClass,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsSessionConfig {
    pub venue: String,
    pub requires_auth: bool,
    pub ack_mode: PrivateWsAckMode,
    pub auth_ack_timeout_ms: u64,
    pub sub_ack_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsSessionPlan {
    pub channels: Vec<PrivateWsChannel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsAuthPlan {
    pub key_id: String,
    pub login_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsResumePlan {
    pub reconnect_attempt: u32,
    pub channels: Vec<PrivateWsChannel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsEventEnvelope {
    pub channel: Option<PrivateWsChannel>,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateWsSession {
    pub config: PrivateWsSessionConfig,
    pub state: PrivateWsLifecycleState,
    pub auth_ack: bool,
    pub sub_ack: bool,
}

pub trait PrivateWsAuthenticator {
    fn build_login_frame(
        &self,
        auth_plan: &PrivateWsAuthPlan,
    ) -> Result<String, PrivateWsRejectClass>;
    fn handle_auth_message(&self, message: &str) -> Result<Option<bool>, PrivateWsRejectClass>;
    fn is_session_ready(&self, message: &str) -> bool;
}

pub trait PrivateWsSubscriber {
    fn build_subscribe_frame(
        &self,
        channel: PrivateWsChannel,
    ) -> Result<String, PrivateWsRejectClass>;
    fn handle_subscribe_ack(&self, message: &str) -> Result<Option<bool>, PrivateWsRejectClass>;
    fn channel_from_message(&self, message: &str) -> Option<PrivateWsChannel>;
}

pub trait PrivateWsNormalizer {
    fn normalize_event(
        &self,
        envelope: &PrivateWsEventEnvelope,
    ) -> Result<CanonicalPrivateWsEvent, PrivateWsRejectClass>;
}

impl PrivateWsSession {
    pub fn new(config: PrivateWsSessionConfig) -> Self {
        Self {
            config,
            state: PrivateWsLifecycleState::Connecting,
            auth_ack: false,
            sub_ack: false,
        }
    }

    pub fn on_connected(&mut self) {
        self.state = PrivateWsLifecycleState::Authenticating;
    }

    pub fn on_auth_ack(&mut self) {
        self.auth_ack = true;
        self.state = PrivateWsLifecycleState::Authenticated;
    }

    pub fn on_subscribe_sent(&mut self) {
        self.state = PrivateWsLifecycleState::Subscribing;
    }

    pub fn on_sub_ack(&mut self) {
        self.sub_ack = true;
        self.state = PrivateWsLifecycleState::Active;
    }

    pub fn on_first_valid_event(&mut self) {
        if matches!(self.config.ack_mode, PrivateWsAckMode::ImplicitObservation) {
            self.sub_ack = true;
            self.state = PrivateWsLifecycleState::Active;
        }
    }

    pub fn on_reauth_required(&mut self) {
        self.state = PrivateWsLifecycleState::ReauthPending;
    }

    pub fn on_reconnect(&mut self) {
        self.state = PrivateWsLifecycleState::ResubscribePending;
        self.sub_ack = false;
    }

    pub fn fail(&mut self) {
        self.state = PrivateWsLifecycleState::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_ack_becomes_active_on_first_event() {
        let mut s = PrivateWsSession::new(PrivateWsSessionConfig {
            venue: "coincheck".into(),
            requires_auth: true,
            ack_mode: PrivateWsAckMode::ImplicitObservation,
            auth_ack_timeout_ms: 5000,
            sub_ack_timeout_ms: 5000,
        });
        s.on_connected();
        s.on_auth_ack();
        s.on_subscribe_sent();
        s.on_first_valid_event();
        assert_eq!(s.state, PrivateWsLifecycleState::Active);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivateRuntimeSignal {
    AuthExpired,
    AckTimeout,
    HeartbeatTimeout,
    TransportClosed,
}

pub fn signal_to_failure(signal: PrivateRuntimeSignal) -> ucel_core::IngestFailureClass {
    match signal {
        PrivateRuntimeSignal::AuthExpired => ucel_core::IngestFailureClass::AuthFailed,
        PrivateRuntimeSignal::AckTimeout => ucel_core::IngestFailureClass::AckTimeout,
        PrivateRuntimeSignal::HeartbeatTimeout => ucel_core::IngestFailureClass::HeartbeatTimeout,
        PrivateRuntimeSignal::TransportClosed => ucel_core::IngestFailureClass::TransportClosed,
    }
}
