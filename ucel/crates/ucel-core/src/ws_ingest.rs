use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IngestStreamKey {
    pub exchange: String,
    pub family: String,
    pub channel: String,
    pub symbol: String,
    pub shard: u16,
    pub auth_scope: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestLifecycleState {
    Planned,
    PendingConnect,
    Connecting,
    AwaitingAuth,
    AwaitingAck,
    Active,
    StallSuspected,
    ReconnectScheduled,
    ResumePending,
    Deadlettered,
    Drained,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestFailureClass {
    PolicyBlocked,
    AuthFailed,
    AckTimeout,
    HeartbeatTimeout,
    ChecksumMismatch,
    GapDetected,
    RateLimited,
    TransportClosed,
    ParseFailed,
    SpecViolation,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestIntegrityMode {
    None,
    Sequence,
    Checksum,
    SequenceAndChecksum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestRetryBudget {
    pub max_retries: u32,
    pub max_checksum_retries: u32,
    pub max_heartbeat_retries: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestHeartbeatPolicy {
    pub ping_interval_ms: u64,
    pub idle_timeout_ms: u64,
    pub stall_after_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestResumeDirective {
    ReconnectOnly,
    Resubscribe,
    ResnapshotThenResubscribe,
    ReauthThenResubscribe,
    Deadletter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IngestCheckpoint {
    pub last_event_ts: Option<u64>,
    pub last_sequence: Option<u64>,
    pub checksum: Option<String>,
    pub active_since: Option<u64>,
    pub metadata: serde_json::Value,
}

pub fn is_valid_transition(from: IngestLifecycleState, to: IngestLifecycleState) -> bool {
    use IngestLifecycleState::*;
    matches!(
        (from, to),
        (Planned, PendingConnect)
            | (PendingConnect, Connecting)
            | (Connecting, AwaitingAuth)
            | (Connecting, AwaitingAck)
            | (AwaitingAuth, AwaitingAck)
            | (AwaitingAck, Active)
            | (Active, StallSuspected)
            | (Active, ReconnectScheduled)
            | (StallSuspected, ReconnectScheduled)
            | (ReconnectScheduled, ResumePending)
            | (ResumePending, PendingConnect)
            | (_, Deadlettered)
            | (_, Drained)
            | (Drained, Completed)
    )
}

pub fn failure_to_resume_directive(
    failure: IngestFailureClass,
    integrity_mode: IngestIntegrityMode,
) -> IngestResumeDirective {
    use IngestFailureClass::*;
    match failure {
        PolicyBlocked | SpecViolation => IngestResumeDirective::Deadletter,
        AuthFailed => IngestResumeDirective::ReauthThenResubscribe,
        ChecksumMismatch | GapDetected => {
            if matches!(
                integrity_mode,
                IngestIntegrityMode::Checksum | IngestIntegrityMode::SequenceAndChecksum
            ) {
                IngestResumeDirective::ResnapshotThenResubscribe
            } else {
                IngestResumeDirective::Resubscribe
            }
        }
        AckTimeout | HeartbeatTimeout | RateLimited | TransportClosed | ParseFailed | Shutdown => {
            IngestResumeDirective::Resubscribe
        }
    }
}

pub fn escalate_integrity_failure(
    retries: u32,
    budget: IngestRetryBudget,
    failure: IngestFailureClass,
) -> bool {
    match failure {
        IngestFailureClass::ChecksumMismatch => retries > budget.max_checksum_retries,
        IngestFailureClass::HeartbeatTimeout => retries > budget.max_heartbeat_retries,
        _ => retries > budget.max_retries,
    }
}
