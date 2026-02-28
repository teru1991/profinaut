pub mod events;
pub mod metrics;

use tracing::info;

use crate::stability::events::{
    BreakerState, ConnState, OutqOutcome, ReconnectReason, ShutdownPhase, TransportStabilityEvent,
};
use crate::stability::metrics::StabilityMetrics;

#[derive(Debug, Default)]
pub struct StabilityHub {
    metrics: StabilityMetrics,
}

impl StabilityHub {
    pub fn new() -> Self {
        Self {
            metrics: StabilityMetrics::new(),
        }
    }

    pub fn set_gauge(&self, name: &str, value: i64) {
        self.metrics.set_gauge(name, value);
    }

    pub fn add_gauge(&self, name: &str, by: i64) {
        self.metrics.add_gauge(name, by);
    }

    pub fn emit(&self, event: TransportStabilityEvent) {
        match event {
            TransportStabilityEvent::ReconnectAttempt {
                exchange_id,
                conn_id,
                reason,
                attempt,
            } => {
                info!(event_kind="ReconnectAttempt", exchange_id=%exchange_id, conn_id=%conn_id, reason=?reason, attempt=attempt);
                self.metrics.inc_counter(
                    "reconnect_attempt_total",
                    &format!("reason={:?}", reason),
                    1,
                );
            }
            TransportStabilityEvent::CircuitBreakerState {
                exchange_id,
                conn_id,
                state,
            } => {
                info!(event_kind="CircuitBreakerState", exchange_id=%exchange_id, conn_id=%conn_id, state=?state);
                self.metrics.inc_counter(
                    "breaker_state_change_total",
                    &format!("state={:?}", state),
                    1,
                );
            }
            TransportStabilityEvent::StaleRequeued {
                exchange_id,
                conn_id,
                count,
            } => {
                info!(event_kind="StaleRequeued", exchange_id=%exchange_id, conn_id=%conn_id, count=count);
                self.metrics
                    .inc_counter("stale_requeued_total", "all", count);
            }
            TransportStabilityEvent::OutqOverflowOutcome {
                exchange_id,
                conn_id,
                outcome,
            } => {
                info!(event_kind="OutqOverflowOutcome", exchange_id=%exchange_id, conn_id=%conn_id, outcome=?outcome);
                self.metrics.inc_counter(
                    "outq_overflow_total",
                    &format!("outcome={:?}", outcome),
                    1,
                );
            }
            TransportStabilityEvent::RlPenaltyApplied {
                exchange_id,
                conn_id,
                priority,
                penalty_ms,
            } => {
                info!(event_kind="RlPenaltyApplied", exchange_id=%exchange_id, conn_id=%conn_id, priority=%priority.as_str(), penalty_ms=penalty_ms);
                self.metrics.inc_counter(
                    "rl_penalty_applied_ms_total",
                    &format!("priority={}", priority.as_str()),
                    penalty_ms,
                );
                self.metrics.inc_counter(
                    "rl_penalty_applied_count",
                    &format!("priority={}", priority.as_str()),
                    1,
                );
            }
            TransportStabilityEvent::RlCooldownSet {
                exchange_id,
                conn_id,
                priority,
                cooldown_secs,
                attempts,
            } => {
                info!(event_kind="RlCooldownSet", exchange_id=%exchange_id, conn_id=%conn_id, priority=%priority.as_str(), cooldown_secs=cooldown_secs, attempts=attempts);
                self.metrics.inc_counter(
                    "rl_cooldown_set_total",
                    &format!("priority={}", priority.as_str()),
                    1,
                );
            }
            TransportStabilityEvent::ConnectionState {
                exchange_id,
                conn_id,
                state,
            } => {
                info!(event_kind="ConnectionState", exchange_id=%exchange_id, conn_id=%conn_id, state=?state);
            }
            TransportStabilityEvent::ShutdownPhase {
                exchange_id,
                conn_id,
                phase,
            } => {
                info!(event_kind="ShutdownPhase", exchange_id=%exchange_id, conn_id=%conn_id, phase=?phase);
                if matches!(phase, ShutdownPhase::AbortTimeout) {
                    self.metrics.inc_counter("shutdown_abort_total", "all", 1);
                }
            }
        }
    }
}

pub fn map_breaker_state(s: crate::ws::circuit_breaker::CircuitStateKind) -> BreakerState {
    match s {
        crate::ws::circuit_breaker::CircuitStateKind::Closed => BreakerState::Closed,
        crate::ws::circuit_breaker::CircuitStateKind::HalfOpen => BreakerState::HalfOpen,
        crate::ws::circuit_breaker::CircuitStateKind::Open => BreakerState::Open,
    }
}

pub fn map_outcome(o: crate::ws::priority::PushOutcome) -> OutqOutcome {
    match o {
        crate::ws::priority::PushOutcome::Enqueued => OutqOutcome::Enqueued,
        crate::ws::priority::PushOutcome::Dropped => OutqOutcome::Dropped,
        crate::ws::priority::PushOutcome::Spilled => OutqOutcome::Spilled,
    }
}

pub fn _map_conn_state(c: ConnState) -> ConnState {
    c
}

pub fn _map_reconnect_reason(r: ReconnectReason) -> ReconnectReason {
    r
}
