/// Heartbeat + stale detection config.
///
/// Heartbeat covers *connection-level* liveness (idle timeout).
/// Stale detection covers *subscription-level* liveness (no messages per subscription).

#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    /// How often we should send ping (writer side).
    pub ping_interval_secs: u64,
    /// If no message is received for this many seconds, connection is considered idle/stale.
    pub idle_timeout_secs: u64,
}

pub fn is_idle(last_message_age_secs: u64, cfg: HeartbeatConfig) -> bool {
    last_message_age_secs >= cfg.idle_timeout_secs
}

/// Subscription stale detection config (spec-fixed).
///
/// Used to detect "active but stale" subscriptions and requeue them to pending
/// so the system can auto-resubscribe.
#[derive(Debug, Clone, Copy)]
pub struct StaleConfig {
    /// A subscription is considered stale if no message is observed for > stale_after_secs.
    ///
    /// For subscriptions that have never received a message (`last_message_at` NULL),
    /// `first_active_at` is used as the fallback timestamp.
    pub stale_after_secs: i64,

    /// Max number of subscriptions to requeue per sweep (storm guard for requeue).
    pub max_batch: usize,
}

impl Default for StaleConfig {
    fn default() -> Self {
        Self {
            stale_after_secs: 60,
            max_batch: 200,
        }
    }
}

/// Compute cutoff timestamp (unix seconds) for stale detection.
///
/// `now_unix` is unix seconds.
/// The cutoff is `now_unix - stale_after_secs`.
pub fn stale_cutoff_unix(now_unix: i64, cfg: StaleConfig) -> i64 {
    now_unix.saturating_sub(cfg.stale_after_secs.max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_logic_is_stable() {
        let cfg = HeartbeatConfig {
            ping_interval_secs: 10,
            idle_timeout_secs: 30,
        };
        assert!(!is_idle(29, cfg));
        assert!(is_idle(30, cfg));
    }

    #[test]
    fn cutoff_is_now_minus_stale_after() {
        let cfg = StaleConfig {
            stale_after_secs: 30,
            max_batch: 100,
        };
        assert_eq!(stale_cutoff_unix(100, cfg), 70);
    }
}
