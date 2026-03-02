use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    pub ping_interval_secs: u64,
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct HeartbeatTracker {
    cfg: HeartbeatConfig,
    last_recv: Instant,
}

impl HeartbeatTracker {
    pub fn new(cfg: HeartbeatConfig, now: Instant) -> Self {
        Self {
            cfg,
            last_recv: now,
        }
    }

    pub fn observe_recv(&mut self, now: Instant) {
        self.last_recv = now;
    }

    pub fn last_recv(&self) -> Instant {
        self.last_recv
    }

    pub fn is_stale(&self, now: Instant) -> bool {
        now.duration_since(self.last_recv).as_secs() >= self.cfg.idle_timeout_secs
    }

    pub fn next_ping_after(&self, now: Instant) -> Duration {
        let elapsed = now.duration_since(self.last_recv);
        let ping = Duration::from_secs(self.cfg.ping_interval_secs);
        if elapsed >= ping {
            Duration::ZERO
        } else {
            ping - elapsed
        }
    }
}

pub fn is_idle(last_message_age_secs: u64, cfg: HeartbeatConfig) -> bool {
    last_message_age_secs >= cfg.idle_timeout_secs
}

#[derive(Debug, Clone, Copy)]
pub struct StaleConfig {
    pub stale_after_secs: i64,
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

pub fn stale_cutoff_unix(now_unix: i64, cfg: StaleConfig) -> i64 {
    now_unix.saturating_sub(cfg.stale_after_secs.max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_detects_stale() {
        let cfg = HeartbeatConfig {
            ping_interval_secs: 1,
            idle_timeout_secs: 2,
        };
        let start = Instant::now();
        let hb = HeartbeatTracker::new(cfg, start);
        assert!(!hb.is_stale(start + Duration::from_secs(1)));
        assert!(hb.is_stale(start + Duration::from_secs(2)));
    }
}
