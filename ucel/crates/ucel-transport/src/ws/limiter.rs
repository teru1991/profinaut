use std::time::{Duration, Instant};

use super::priority::OutboundPriority;

/// Token-bucket config for WS outbound.
///
/// - capacity: max burst tokens
/// - refill_per_sec: steady refill
#[derive(Debug, Clone, Copy)]
pub struct BucketConfig {
    pub capacity: f64,
    pub refill_per_sec: f64,
}

impl BucketConfig {
    pub fn per_second(rps: f64) -> Self {
        let rps = rps.max(0.01);
        Self {
            capacity: rps.max(1.0),
            refill_per_sec: rps,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WsRateLimiterConfig {
    pub control: BucketConfig,
    pub private: BucketConfig,
    pub public: BucketConfig,

    /// Optional global floor interval to prevent micro-bursts.
    pub min_gap: Duration,
}

impl Default for WsRateLimiterConfig {
    fn default() -> Self {
        Self {
            control: BucketConfig::per_second(5.0),
            private: BucketConfig::per_second(2.0),
            public: BucketConfig::per_second(1.0),
            min_gap: Duration::from_millis(0),
        }
    }
}

#[derive(Debug, Clone)]
struct TokenBucket {
    cap: f64,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,

    // penalty window (retry-after style)
    penalty_until: Option<Instant>,
}

impl TokenBucket {
    fn new(cfg: BucketConfig) -> Self {
        Self {
            cap: cfg.capacity.max(0.01),
            refill_per_sec: cfg.refill_per_sec.max(0.01),
            tokens: cfg.capacity.max(0.01),
            last: Instant::now(),
            penalty_until: None,
        }
    }

    fn apply_penalty(&mut self, now: Instant, d: Duration) {
        let until = now + d;
        self.penalty_until = Some(match self.penalty_until {
            Some(prev) if prev > until => prev,
            _ => until,
        });
    }

    fn advance(&mut self, now: Instant) {
        let dt = now.duration_since(self.last);
        let add = dt.as_secs_f64() * self.refill_per_sec;
        self.tokens = (self.tokens + add).min(self.cap);
        self.last = now;
    }

    /// Acquire one token; returns required wait.
    fn take_one(&mut self, now: Instant) -> Duration {
        if let Some(until) = self.penalty_until {
            if now < until {
                return until.duration_since(now);
            }
        }
        self.advance(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Duration::from_secs(0)
        } else {
            let needed = 1.0 - self.tokens;
            let secs = needed / self.refill_per_sec;
            Duration::from_secs_f64(secs.max(0.0))
        }
    }
}

#[derive(Debug)]
pub struct WsRateLimiter {
    cfg: WsRateLimiterConfig,
    control: TokenBucket,
    private: TokenBucket,
    public: TokenBucket,

    // small global pacing
    last_grant: Instant,
}

impl WsRateLimiter {
    pub fn new(cfg: WsRateLimiterConfig) -> Self {
        Self {
            cfg,
            control: TokenBucket::new(cfg.control),
            private: TokenBucket::new(cfg.private),
            public: TokenBucket::new(cfg.public),
            last_grant: Instant::now(),
        }
    }

    fn bucket_mut(&mut self, p: OutboundPriority) -> &mut TokenBucket {
        match p {
            OutboundPriority::Control => &mut self.control,
            OutboundPriority::Private => &mut self.private,
            OutboundPriority::Public => &mut self.public,
        }
    }

    /// Acquire permission for a priority class.
    ///
    /// Returns how long caller should wait before sending.
    pub fn acquire_wait(&mut self, p: OutboundPriority, now: Instant) -> Duration {
        // Optional global min-gap
        let mut wait = Duration::from_secs(0);
        if self.cfg.min_gap > Duration::from_secs(0) {
            let gap = now.duration_since(self.last_grant);
            if gap < self.cfg.min_gap {
                wait = self.cfg.min_gap - gap;
            }
        }

        // Priority-specific bucket
        let bw = self.bucket_mut(p).take_one(now);
        wait = wait.max(bw);

        if wait == Duration::from_secs(0) {
            self.last_grant = now;
        }
        wait
    }

    /// Apply a penalty window (retry-after style) to a bucket.
    ///
    /// Example use:
    /// - If server responds with "rate limit" for private channel, call:
    ///   apply_penalty(Private, Duration::from_secs(1))
    pub fn apply_penalty(&mut self, p: OutboundPriority, now: Instant, d: Duration) {
        self.bucket_mut(p).apply_penalty(now, d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn buckets_are_separate() {
        let mut lim = WsRateLimiter::new(WsRateLimiterConfig {
            control: BucketConfig { capacity: 1.0, refill_per_sec: 1.0 },
            private: BucketConfig { capacity: 1.0, refill_per_sec: 1.0 },
            public: BucketConfig { capacity: 1.0, refill_per_sec: 1.0 },
            min_gap: Duration::from_millis(0),
        });

        let t0 = Instant::now();
        assert_eq!(lim.acquire_wait(OutboundPriority::Public, t0), Duration::from_secs(0));
        assert!(lim.acquire_wait(OutboundPriority::Public, t0) > Duration::from_millis(0));

        // Private bucket is separate => still grants
        assert_eq!(lim.acquire_wait(OutboundPriority::Private, t0), Duration::from_secs(0));
    }

    #[test]
    fn penalty_forces_wait() {
        let mut lim = WsRateLimiter::new(WsRateLimiterConfig::default());
        let t0 = Instant::now();
        lim.apply_penalty(OutboundPriority::Private, t0, Duration::from_millis(50));
        assert!(lim.acquire_wait(OutboundPriority::Private, t0) >= Duration::from_millis(50));
    }
}