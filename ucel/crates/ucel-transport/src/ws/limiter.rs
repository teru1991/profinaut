use std::time::{Duration, Instant};

use super::priority::OutboundPriority;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct ThrottleCounters {
    pub control_events: u64,
    pub private_events: u64,
    pub public_events: u64,
    pub forced_gate_events: u64,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    cap: f64,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,
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

    fn take_one(&mut self, now: Instant) -> Duration {
        if let Some(until) = self.penalty_until {
            if now < until {
                return until.duration_since(now);
            }
        }
        self.advance(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64((needed / self.refill_per_sec).max(0.0))
        }
    }
}

#[derive(Debug)]
pub struct WsRateLimiter {
    cfg: WsRateLimiterConfig,
    control: TokenBucket,
    private: TokenBucket,
    public: TokenBucket,
    last_grant: Instant,
    forced_gate_until: Option<Instant>,
    counters: ThrottleCounters,
}

impl WsRateLimiter {
    pub fn new(cfg: WsRateLimiterConfig) -> Self {
        Self {
            cfg,
            control: TokenBucket::new(cfg.control),
            private: TokenBucket::new(cfg.private),
            public: TokenBucket::new(cfg.public),
            last_grant: Instant::now(),
            forced_gate_until: None,
            counters: ThrottleCounters::default(),
        }
    }

    fn bucket_mut(&mut self, p: OutboundPriority) -> &mut TokenBucket {
        match p {
            OutboundPriority::Control => &mut self.control,
            OutboundPriority::Private => &mut self.private,
            OutboundPriority::Public => &mut self.public,
        }
    }

    pub fn acquire_wait(&mut self, p: OutboundPriority, now: Instant) -> Duration {
        if let Some(until) = self.forced_gate_until {
            if now < until {
                self.counters.forced_gate_events += 1;
                return until.duration_since(now);
            }
        }

        let mut wait = Duration::ZERO;
        if self.cfg.min_gap > Duration::ZERO {
            let gap = now.duration_since(self.last_grant);
            if gap < self.cfg.min_gap {
                wait = self.cfg.min_gap - gap;
            }
        }

        let bw = self.bucket_mut(p).take_one(now);
        wait = wait.max(bw);

        if wait == Duration::ZERO {
            self.last_grant = now;
        } else {
            match p {
                OutboundPriority::Control => self.counters.control_events += 1,
                OutboundPriority::Private => self.counters.private_events += 1,
                OutboundPriority::Public => self.counters.public_events += 1,
            }
        }

        wait
    }

    pub fn apply_penalty(&mut self, p: OutboundPriority, now: Instant, d: Duration) {
        self.bucket_mut(p).apply_penalty(now, d);
    }

    pub fn apply_retry_after_gate(&mut self, now: Instant, retry_after: Duration) {
        let next_until = now + retry_after;
        self.forced_gate_until = Some(match self.forced_gate_until {
            Some(prev) if prev > next_until => prev,
            _ => next_until,
        });
    }

    pub fn counters(&self) -> ThrottleCounters {
        self.counters
    }
}
