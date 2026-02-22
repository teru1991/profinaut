use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TokenLimiter {
    max_per_window: u64,
    window: Duration,
    count: u64,
    started_at: Instant,
}

impl TokenLimiter {
    pub fn per_second(limit: u64) -> Self {
        Self { max_per_window: limit.max(1), window: Duration::from_secs(1), count: 0, started_at: Instant::now() }
    }

    pub fn per_hour(limit: u64) -> Self {
        Self { max_per_window: limit.max(1), window: Duration::from_secs(3600), count: 0, started_at: Instant::now() }
    }

    pub fn allow(&mut self, now: Instant) -> bool {
        if now.duration_since(self.started_at) >= self.window {
            self.started_at = now;
            self.count = 0;
        }
        if self.count < self.max_per_window {
            self.count += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct TwoLevelLimiter {
    pub global: TokenLimiter,
    pub conn: TokenLimiter,
}

impl TwoLevelLimiter {
    pub fn allow(&mut self, now: Instant) -> bool {
        self.global.allow(now) && self.conn.allow(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limiter_blocks_when_limit_hit() {
        let mut l = TwoLevelLimiter { global: TokenLimiter::per_second(1), conn: TokenLimiter::per_second(2) };
        let now = Instant::now();
        assert!(l.allow(now));
        assert!(!l.allow(now));
    }
}
