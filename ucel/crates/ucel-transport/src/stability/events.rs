use crate::ws::priority::OutboundPriority;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectReason {
    ConnectError,
    ConnectTimeout,
    IdleTimeout,
    MaxAge,
    ReadError,
    CloseFrame,
    CircuitOpenWait,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    HalfOpen,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutqOutcome {
    Enqueued,
    Dropped,
    Spilled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Connected,
    Disconnected,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    CloseRequested,
    Flushing,
    Requeueing,
    Joined,
    AbortTimeout,
}

#[derive(Debug, Clone)]
pub enum TransportStabilityEvent {
    ReconnectAttempt {
        exchange_id: String,
        conn_id: String,
        reason: ReconnectReason,
        attempt: u64,
    },
    CircuitBreakerState {
        exchange_id: String,
        conn_id: String,
        state: BreakerState,
    },
    StaleRequeued {
        exchange_id: String,
        conn_id: String,
        count: u64,
    },
    OutqOverflowOutcome {
        exchange_id: String,
        conn_id: String,
        outcome: OutqOutcome,
    },
    RlPenaltyApplied {
        exchange_id: String,
        conn_id: String,
        priority: OutboundPriority,
        penalty_ms: u64,
    },
    RlCooldownSet {
        exchange_id: String,
        conn_id: String,
        priority: OutboundPriority,
        cooldown_secs: i64,
        attempts: i64,
    },
    ConnectionState {
        exchange_id: String,
        conn_id: String,
        state: ConnState,
    },
    ShutdownPhase {
        exchange_id: String,
        conn_id: String,
        phase: ShutdownPhase,
    },
}
