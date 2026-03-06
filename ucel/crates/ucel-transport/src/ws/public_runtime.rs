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

#[derive(Debug, Clone)]
pub struct PublicWsSession {
    pub config: PublicWsSessionConfig,
    pub inflight: Vec<PublicWsSubscribePlan>,
    pub active: Vec<PublicWsSubscribePlan>,
}

impl PublicWsSession {
    pub fn new(config: PublicWsSessionConfig) -> Self {
        Self {
            config,
            inflight: Vec::new(),
            active: Vec::new(),
        }
    }

    pub fn mark_subscribing(&mut self, plan: PublicWsSubscribePlan) {
        self.inflight.push(plan);
    }

    pub fn mark_active(&mut self, channel: &str, symbol: &str) {
        if let Some(i) = self
            .inflight
            .iter()
            .position(|p| p.channel == channel && p.symbol == symbol)
        {
            let plan = self.inflight.remove(i);
            self.active.push(plan);
        }
    }

    pub fn resume_plan(&self) -> PublicWsResumePlan {
        PublicWsResumePlan {
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
