use crate::health::{Health, HealthState};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Supervisor {
    workers: BTreeMap<String, Health>,
}

impl Supervisor {
    pub fn upsert_exchange(&mut self, exchange: impl Into<String>) {
        self.workers.entry(exchange.into()).or_default();
    }

    pub fn mark_exchange_reason(&mut self, exchange: &str, reason: &str) {
        self.workers
            .entry(exchange.to_string())
            .or_default()
            .mark_reason(reason.to_string());
    }

    pub fn state_for(&self, exchange: &str) -> Option<HealthState> {
        self.workers.get(exchange).map(|h| h.state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthState;
    use crate::rest_worker::{handle_rest_failure, handle_rest_success};
    use crate::resync::ResyncController;
    use crate::ws_worker::{handle_ws_signal, WsSignal};

    #[test]
    fn bulkhead_isolated_per_exchange() {
        let mut supervisor = Supervisor::default();
        supervisor.upsert_exchange("binance");
        supervisor.upsert_exchange("bybit");

        supervisor.mark_exchange_reason("binance", "rest_failed");

        assert_eq!(supervisor.state_for("binance"), Some(HealthState::Degraded));
        assert_eq!(supervisor.state_for("bybit"), Some(HealthState::Healthy));
    }

    #[test]
    fn degraded_transition_by_rest_and_ws() {
        let mut health = Health::default();
        handle_rest_failure(&mut health);
        assert_eq!(health.state(), HealthState::Degraded);
        handle_rest_success(&mut health);
        assert_eq!(health.state(), HealthState::Healthy);

        handle_ws_signal(&mut health, WsSignal::Lagged);
        assert_eq!(health.state(), HealthState::Degraded);
        handle_ws_signal(&mut health, WsSignal::Reconnected);
        assert_eq!(health.state(), HealthState::Healthy);
    }

    #[test]
    fn lagged_triggers_resync_clear_stale() {
        let mut resync = ResyncController::default();
        resync.on_restore();
        assert!(resync.stale);

        let snapshot = ucel_symbol_core::Snapshot::new_rest(vec![]);
        resync.on_fresh_snapshot(&snapshot);
        assert!(!resync.stale);
    }
}
