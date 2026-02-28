//! WS reconnect circuit breaker.
//!
//! Goals:
//! - Prevent infinite rapid reconnect loops.
//! - Separate concerns from `storm_guard` (short window count).
//! - Provide deterministic state transitions: Closed -> Open -> HalfOpen -> Closed.
//!
//! NOTE:
//! - `storm_guard` is a *safety fuse* for bursts.
//! - This breaker is a *health gate* for persistent failures.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures in `Closed` before opening.
    pub failure_threshold: u32,
    /// Number of consecutive successes in `HalfOpen` required to close.
    pub success_threshold: u32,
    /// Cooldown duration while `Open`.
    pub cooldown: Duration,
    /// Maximum trial attempts in `HalfOpen` before re-opening.
    pub half_open_max_trials: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            cooldown: Duration::from_secs(20),
            half_open_max_trials: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitStateKind {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed { consecutive_failures: u32 },
    Open { opened_at: Instant },
    HalfOpen { trials: u32, successes: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitDecision {
    /// Attempt is allowed now.
    Allow,
    /// Attempt is not allowed yet; wait this long and try again.
    Wait(Duration),
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    cfg: CircuitBreakerConfig,
    state: CircuitState,
}

impl CircuitBreaker {
    pub fn new(cfg: CircuitBreakerConfig) -> Self {
        let cfg = CircuitBreakerConfig {
            failure_threshold: cfg.failure_threshold.max(1),
            success_threshold: cfg.success_threshold.max(1),
            cooldown: cfg.cooldown.max(Duration::from_millis(1)),
            half_open_max_trials: cfg.half_open_max_trials.max(1),
        };
        Self {
            cfg,
            state: CircuitState::Closed {
                consecutive_failures: 0,
            },
        }
    }

    pub fn kind(&self) -> CircuitStateKind {
        match self.state {
            CircuitState::Closed { .. } => CircuitStateKind::Closed,
            CircuitState::Open { .. } => CircuitStateKind::Open,
            CircuitState::HalfOpen { .. } => CircuitStateKind::HalfOpen,
        }
    }

    pub fn reset(&mut self) {
        self.state = CircuitState::Closed {
            consecutive_failures: 0,
        };
    }

    /// Decide whether a new attempt may start *now*.
    ///
    /// This method may transition `Open -> HalfOpen` when cooldown has elapsed.
    pub fn before_attempt(&mut self, now: Instant) -> CircuitDecision {
        match self.state {
            CircuitState::Closed { .. } => CircuitDecision::Allow,
            CircuitState::HalfOpen { trials, .. } => {
                if trials >= self.cfg.half_open_max_trials {
                    // Too many trials without closing => back to open.
                    self.state = CircuitState::Open { opened_at: now };
                    CircuitDecision::Wait(self.cfg.cooldown)
                } else {
                    CircuitDecision::Allow
                }
            }
            CircuitState::Open { opened_at } => {
                let elapsed = now.duration_since(opened_at);
                if elapsed >= self.cfg.cooldown {
                    self.state = CircuitState::HalfOpen {
                        trials: 0,
                        successes: 0,
                    };
                    CircuitDecision::Allow
                } else {
                    CircuitDecision::Wait(self.cfg.cooldown.saturating_sub(elapsed))
                }
            }
        }
    }

    /// Record a successful attempt.
    pub fn on_success(&mut self, now: Instant) {
        match self.state {
            CircuitState::Closed { .. } => {
                // Any success keeps closed and clears failure streak.
                self.state = CircuitState::Closed {
                    consecutive_failures: 0,
                };
            }
            CircuitState::Open { .. } => {
                // A "success" while open implies external state changed.
                // Move to half-open to validate sustained recovery.
                self.state = CircuitState::HalfOpen {
                    trials: 0,
                    successes: 1,
                };
            }
            CircuitState::HalfOpen { trials, successes } => {
                let successes = successes.saturating_add(1);
                let trials = trials.max(1);
                if successes >= self.cfg.success_threshold {
                    self.state = CircuitState::Closed {
                        consecutive_failures: 0,
                    };
                } else {
                    self.state = CircuitState::HalfOpen { trials, successes };
                }
            }
        }

        // Ensure `now` is used (future extensions / lint hygiene)
        let _ = now;
    }

    /// Record a failed attempt.
    pub fn on_failure(&mut self, now: Instant) {
        match self.state {
            CircuitState::Closed {
                consecutive_failures,
            } => {
                let cf = consecutive_failures.saturating_add(1);
                if cf >= self.cfg.failure_threshold {
                    self.state = CircuitState::Open { opened_at: now };
                } else {
                    self.state = CircuitState::Closed {
                        consecutive_failures: cf,
                    };
                }
            }
            CircuitState::Open { .. } => {
                // Refresh open window (keeps it open longer).
                self.state = CircuitState::Open { opened_at: now };
            }
            CircuitState::HalfOpen { .. } => {
                // Any failure in half-open re-opens immediately.
                self.state = CircuitState::Open { opened_at: now };
            }
        }
    }

    /// Record that a half-open attempt has started.
    pub fn on_half_open_trial(&mut self) {
        if let CircuitState::HalfOpen { trials, successes } = self.state {
            self.state = CircuitState::HalfOpen {
                trials: trials.saturating_add(1),
                successes,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_consecutive_failures_and_cools_down() {
        let mut b = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            cooldown: Duration::from_millis(100),
            half_open_max_trials: 2,
        });

        let t0 = Instant::now();
        assert_eq!(b.before_attempt(t0), CircuitDecision::Allow);
        b.on_failure(t0);
        assert_eq!(b.kind(), CircuitStateKind::Closed);

        let t1 = t0 + Duration::from_millis(1);
        b.on_failure(t1);
        assert_eq!(b.kind(), CircuitStateKind::Open);

        // still cooling
        let t2 = t1 + Duration::from_millis(50);
        match b.before_attempt(t2) {
            CircuitDecision::Wait(d) => assert!(d > Duration::from_millis(0)),
            CircuitDecision::Allow => panic!("should be waiting"),
        }

        // cooldown elapsed => half-open
        let t3 = t1 + Duration::from_millis(120);
        assert_eq!(b.before_attempt(t3), CircuitDecision::Allow);
        assert_eq!(b.kind(), CircuitStateKind::HalfOpen);
        b.on_half_open_trial();
        b.on_success(t3);
        assert_eq!(b.kind(), CircuitStateKind::Closed);
    }

    #[test]
    fn half_open_failure_reopens() {
        let mut b = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            cooldown: Duration::from_millis(10),
            half_open_max_trials: 3,
        });

        let t0 = Instant::now();
        b.on_failure(t0);
        assert_eq!(b.kind(), CircuitStateKind::Open);

        // enter half-open
        let t1 = t0 + Duration::from_millis(20);
        assert_eq!(b.before_attempt(t1), CircuitDecision::Allow);
        assert_eq!(b.kind(), CircuitStateKind::HalfOpen);

        b.on_half_open_trial();
        b.on_failure(t1);
        assert_eq!(b.kind(), CircuitStateKind::Open);
    }
}
