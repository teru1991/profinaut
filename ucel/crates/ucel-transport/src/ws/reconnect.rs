use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectCircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    pub base_ms: u64,
    pub max_ms: u64,
    pub jitter_ms: u64,
    pub storm_window: Duration,
    pub storm_max_failures: usize,
    pub circuit_open_ms: u64,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            base_ms: 200,
            max_ms: 30_000,
            jitter_ms: 250,
            storm_window: Duration::from_secs(30),
            storm_max_failures: 10,
            circuit_open_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReconnectGuard {
    policy: ReconnectPolicy,
    state: ReconnectCircuitState,
    fail_window: VecDeque<Duration>,
    open_until: Option<Duration>,
    jitter_seed: u64,
}

impl ReconnectGuard {
    pub fn new(policy: ReconnectPolicy, seed: u64) -> Self {
        Self {
            policy,
            state: ReconnectCircuitState::Closed,
            fail_window: VecDeque::new(),
            open_until: None,
            jitter_seed: seed.max(1),
        }
    }

    pub fn state(&self) -> ReconnectCircuitState {
        self.state
    }

    pub fn on_failure(&mut self, now: Duration) {
        self.fail_window.push_back(now);
        while let Some(ts) = self.fail_window.front().copied() {
            if now.saturating_sub(ts) > self.policy.storm_window {
                self.fail_window.pop_front();
            } else {
                break;
            }
        }

        if self.fail_window.len() > self.policy.storm_max_failures {
            self.state = ReconnectCircuitState::Open;
            self.open_until = Some(now + Duration::from_millis(self.policy.circuit_open_ms));
        }
    }

    pub fn on_success(&mut self) {
        self.state = ReconnectCircuitState::Closed;
        self.open_until = None;
        self.fail_window.clear();
    }

    pub fn pre_connect_wait(&mut self, now: Duration) -> Duration {
        match self.state {
            ReconnectCircuitState::Closed => Duration::ZERO,
            ReconnectCircuitState::HalfOpen => Duration::ZERO,
            ReconnectCircuitState::Open => {
                if let Some(until) = self.open_until {
                    if now < until {
                        until - now
                    } else {
                        self.state = ReconnectCircuitState::HalfOpen;
                        self.open_until = None;
                        Duration::ZERO
                    }
                } else {
                    self.state = ReconnectCircuitState::HalfOpen;
                    Duration::ZERO
                }
            }
        }
    }

    pub fn next_backoff_ms(&mut self, attempt: u32) -> u64 {
        backoff_with_jitter_ms_seeded(
            attempt,
            self.policy.base_ms,
            self.policy.max_ms,
            self.policy.jitter_ms,
            &mut self.jitter_seed,
        )
    }
}

pub fn backoff_with_jitter_ms(attempt: u32, base_ms: u64, max_ms: u64, jitter_ms: u64) -> u64 {
    let mut seed = ((attempt as u64) << 32) ^ base_ms ^ max_ms ^ jitter_ms ^ 0x9E3779B97F4A7C15;
    backoff_with_jitter_ms_seeded(attempt, base_ms, max_ms, jitter_ms, &mut seed)
}

pub fn backoff_with_jitter_ms_seeded(
    attempt: u32,
    base_ms: u64,
    max_ms: u64,
    jitter_ms: u64,
    seed: &mut u64,
) -> u64 {
    let exp = 2u64.saturating_pow(attempt.min(16));
    let ms = base_ms.saturating_mul(exp).min(max_ms);
    let jitter = if jitter_ms == 0 {
        0
    } else {
        xorshift64(seed) % (jitter_ms.saturating_add(1))
    };
    ms.saturating_add(jitter)
}

fn xorshift64(seed: &mut u64) -> u64 {
    let mut x = (*seed).max(1);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    x
}

pub fn storm_guard(reconnect_count_in_window: usize, max: usize) -> bool {
    reconnect_count_in_window <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_jitter_is_deterministic() {
        let mut seed = 42;
        let a = backoff_with_jitter_ms_seeded(3, 100, 10_000, 100, &mut seed);
        let b = backoff_with_jitter_ms_seeded(3, 100, 10_000, 100, &mut seed);
        assert_ne!(a, b);

        let mut seed2 = 42;
        let a2 = backoff_with_jitter_ms_seeded(3, 100, 10_000, 100, &mut seed2);
        assert_eq!(a, a2);
    }

    #[test]
    fn storm_opens_then_half_open() {
        let mut g = ReconnectGuard::new(
            ReconnectPolicy {
                storm_window: Duration::from_secs(10),
                storm_max_failures: 1,
                circuit_open_ms: 100,
                ..ReconnectPolicy::default()
            },
            7,
        );
        g.on_failure(Duration::from_secs(0));
        g.on_failure(Duration::from_secs(1));
        assert_eq!(g.state(), ReconnectCircuitState::Open);
        assert!(g.pre_connect_wait(Duration::from_secs(1)) > Duration::ZERO);
        assert_eq!(
            g.pre_connect_wait(Duration::from_millis(1_200)),
            Duration::ZERO
        );
        assert_eq!(g.state(), ReconnectCircuitState::HalfOpen);
    }
}
