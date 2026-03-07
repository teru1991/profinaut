use serde_json::Value;
use ucel_core::{
    CanonicalCandle, CanonicalOrderBookDelta, CanonicalOrderBookSnapshot, CanonicalTicker,
    CanonicalTrade, PublicWsAckMode, PublicWsIntegrityMode, PublicWsReasonCode,
};

#[derive(Debug, Clone)]
pub struct PublicWsSessionConfig {
    pub ack_mode: PublicWsAckMode,
    pub integrity_mode: PublicWsIntegrityMode,
    pub heartbeat_timeout_ms: u64,
    pub ack_timeout_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PublicWsIntegrityState {
    pub last_sequence: Option<u64>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublicWsSubscribePlan {
    pub channel: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct PublicWsResumePlan {
    pub subscriptions: Vec<PublicWsSubscribePlan>,
}

#[derive(Debug, Clone)]
pub struct PublicWsEventEnvelope {
    pub channel: String,
    pub symbol: String,
    pub reason: Option<PublicWsReasonCode>,
}

pub trait PublicWsSubscriber {
    fn build_subscribe_frame(&self, plan: &PublicWsSubscribePlan) -> Value;
    fn build_unsubscribe_frame(&self, plan: &PublicWsSubscribePlan) -> Value;
    fn handle_ack_message(&self, message: &Value) -> bool;
}

pub trait PublicWsNormalizer {
    fn normalize_ticker(&self, message: &Value) -> Option<CanonicalTicker>;
    fn normalize_trade(&self, message: &Value) -> Option<CanonicalTrade>;
    fn normalize_orderbook(
        &self,
        message: &Value,
    ) -> Option<(
        Option<CanonicalOrderBookSnapshot>,
        Option<CanonicalOrderBookDelta>,
    )>;
    fn normalize_candle(&self, message: &Value) -> Option<CanonicalCandle>;
}

pub trait PublicWsIntegrityChecker {
    fn apply_snapshot(&mut self, snapshot: &CanonicalOrderBookSnapshot);
    fn apply_delta(&mut self, delta: &CanonicalOrderBookDelta) -> Result<(), PublicWsReasonCode>;
    fn verify_checksum(&self, expected: &str) -> Result<(), PublicWsReasonCode>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomesticPublicWsReadyState {
    Planned,
    Subscribed,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomesticPublicWsDeadletterReason {
    AckTimeout,
    HeartbeatTimeout,
    GapDetected,
    ChecksumMismatch,
    TransportClosed,
}

#[derive(Debug, Clone)]
pub struct DomesticPublicWsSessionConfig {
    pub ack_mode: PublicWsAckMode,
    pub integrity_mode: PublicWsIntegrityMode,
    pub heartbeat_timeout_ms: u64,
    pub ack_timeout_ms: u64,
}

impl From<PublicWsSessionConfig> for DomesticPublicWsSessionConfig {
    fn from(value: PublicWsSessionConfig) -> Self {
        Self {
            ack_mode: value.ack_mode,
            integrity_mode: value.integrity_mode,
            heartbeat_timeout_ms: value.heartbeat_timeout_ms,
            ack_timeout_ms: value.ack_timeout_ms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomesticPublicWsResumePlan {
    pub subscriptions: Vec<PublicWsSubscribePlan>,
}

#[derive(Debug, Clone)]
pub struct PublicWsSession {
    pub config: PublicWsSessionConfig,
    pub inflight: Vec<PublicWsSubscribePlan>,
    pub active: Vec<PublicWsSubscribePlan>,
    pub ready_state: DomesticPublicWsReadyState,
}

impl PublicWsSession {
    pub fn new(config: PublicWsSessionConfig) -> Self {
        Self {
            config,
            inflight: Vec::new(),
            active: Vec::new(),
            ready_state: DomesticPublicWsReadyState::Planned,
        }
    }

    pub fn mark_subscribing(&mut self, plan: PublicWsSubscribePlan) {
        self.inflight.push(plan);
        self.ready_state = DomesticPublicWsReadyState::Subscribed;
    }

    pub fn mark_active(&mut self, channel: &str, symbol: &str) {
        if let Some(i) = self
            .inflight
            .iter()
            .position(|p| p.channel == channel && p.symbol == symbol)
        {
            let plan = self.inflight.remove(i);
            self.active.push(plan);
            self.ready_state = DomesticPublicWsReadyState::Active;
        }
    }

    pub fn observe_event_ready_if_needed(&mut self) {
        if self.config.ack_mode == PublicWsAckMode::ImplicitObservation
            && !self.inflight.is_empty()
            && self.active.is_empty()
        {
            let first = self.inflight.remove(0);
            self.active.push(first);
            self.ready_state = DomesticPublicWsReadyState::Active;
        }
    }

    pub fn activate_immediately_if_needed(&mut self) {
        if self.config.ack_mode == PublicWsAckMode::None && !self.inflight.is_empty() {
            let remaining = std::mem::take(&mut self.inflight);
            self.active.extend(remaining);
            self.ready_state = DomesticPublicWsReadyState::Active;
        }
    }

    pub fn resume_plan(&self) -> PublicWsResumePlan {
        PublicWsResumePlan {
            subscriptions: self.active.clone(),
        }
    }

    pub fn domestic_resume_plan(&self) -> DomesticPublicWsResumePlan {
        DomesticPublicWsResumePlan {
            subscriptions: self.active.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicRuntimeSignal {
    AckObserved,
    HeartbeatTimeout,
    GapDetected,
    ChecksumMismatch,
    TransportClosed,
}

pub fn signal_to_failure(signal: PublicRuntimeSignal) -> Option<ucel_core::IngestFailureClass> {
    match signal {
        PublicRuntimeSignal::AckObserved => None,
        PublicRuntimeSignal::HeartbeatTimeout => {
            Some(ucel_core::IngestFailureClass::HeartbeatTimeout)
        }
        PublicRuntimeSignal::GapDetected => Some(ucel_core::IngestFailureClass::GapDetected),
        PublicRuntimeSignal::ChecksumMismatch => {
            Some(ucel_core::IngestFailureClass::ChecksumMismatch)
        }
        PublicRuntimeSignal::TransportClosed => {
            Some(ucel_core::IngestFailureClass::TransportClosed)
        }
    }
}

pub fn signal_to_deadletter_reason(
    signal: PublicRuntimeSignal,
) -> Option<DomesticPublicWsDeadletterReason> {
    match signal {
        PublicRuntimeSignal::AckObserved => None,
        PublicRuntimeSignal::HeartbeatTimeout => {
            Some(DomesticPublicWsDeadletterReason::HeartbeatTimeout)
        }
        PublicRuntimeSignal::GapDetected => Some(DomesticPublicWsDeadletterReason::GapDetected),
        PublicRuntimeSignal::ChecksumMismatch => {
            Some(DomesticPublicWsDeadletterReason::ChecksumMismatch)
        }
        PublicRuntimeSignal::TransportClosed => {
            Some(DomesticPublicWsDeadletterReason::TransportClosed)
        }
    }
}
