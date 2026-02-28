//! HTTP rate limiting utilities.
//!
//! Key requirements:
//! - Respect per-venue buckets.
//! - Private (auth/order) requests should have separate capacity and can be prioritized.
//!
//! NOTE: Actual sleep is done by the caller. This module returns the required wait duration.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

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
pub struct VenueLimiterConfig {
    pub public: BucketConfig,
    pub private: BucketConfig,
}

impl Default for VenueLimiterConfig {
    fn default() -> Self {
        Self {
            public: BucketConfig::per_second(5.0),
            private: BucketConfig::per_second(2.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRateLimiterConfig {
    pub default: VenueLimiterConfig,
    pub per_venue: HashMap<String, VenueLimiterConfig>,
}

impl Default for HttpRateLimiterConfig {
    fn default() -> Self {
        Self {
            default: VenueLimiterConfig::default(),
            per_venue: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BucketKind {
    Public,
    Private,
}

#[derive(Debug)]
struct TokenBucket {
    cap: f64,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,
}

impl TokenBucket {
    fn new(cfg: BucketConfig) -> Self {
        Self {
            cap: cfg.capacity.max(0.01),
            refill_per_sec: cfg.refill_per_sec.max(0.01),
            tokens: cfg.capacity.max(0.01),
            last: Instant::now(),
        }
    }

    fn advance(&mut self, now: Instant) {
        let dt = now.duration_since(self.last);
        let add = dt.as_secs_f64() * self.refill_per_sec;
        self.tokens = (self.tokens + add).min(self.cap);
        self.last = now;
    }

    fn take_one(&mut self, now: Instant) -> Duration {
        self.advance(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            return Duration::from_secs(0);
        }
        let needed = 1.0 - self.tokens;
        let secs = needed / self.refill_per_sec;
        Duration::from_secs_f64(secs.max(0.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    venue: String,
    kind: BucketKind,
}

#[derive(Debug)]
pub struct HttpRateLimiter {
    cfg: HttpRateLimiterConfig,
    buckets: Mutex<HashMap<Key, TokenBucket>>,
}

impl HttpRateLimiter {
    pub fn new(cfg: HttpRateLimiterConfig) -> Self {
        Self {
            cfg,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    fn venue_cfg(&self, venue: &str) -> VenueLimiterConfig {
        self.cfg
            .per_venue
            .get(venue)
            .copied()
            .unwrap_or(self.cfg.default)
    }

    /// Returns required wait duration to respect the limiter.
    pub async fn acquire_wait(&self, venue: &str, requires_auth: bool, now: Instant) -> Duration {
        let kind = if requires_auth {
            BucketKind::Private
        } else {
            BucketKind::Public
        };
        let key = Key {
            venue: venue.to_string(),
            kind,
        };

        let mut map = self.buckets.lock().await;
        let b = map.entry(key).or_insert_with(|| {
            let vc = self.venue_cfg(venue);
            match kind {
                BucketKind::Public => TokenBucket::new(vc.public),
                BucketKind::Private => TokenBucket::new(vc.private),
            }
        });
        b.take_one(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn token_bucket_waits_when_empty() {
        let mut cfg = HttpRateLimiterConfig::default();
        cfg.default = VenueLimiterConfig {
            public: BucketConfig {
                capacity: 1.0,
                refill_per_sec: 1.0,
            },
            private: BucketConfig::per_second(1.0),
        };
        let lim = HttpRateLimiter::new(cfg);

        let t0 = Instant::now();
        let w0 = lim.acquire_wait("x", false, t0).await;
        assert_eq!(w0, Duration::from_secs(0));

        let w1 = lim.acquire_wait("x", false, t0).await;
        assert!(w1 > Duration::from_secs(0));

        let t1 = t0 + Duration::from_secs(1);
        let w2 = lim.acquire_wait("x", false, t1).await;
        assert_eq!(w2, Duration::from_secs(0));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_and_public_are_separate_buckets() {
        let cfg = HttpRateLimiterConfig::default();
        let lim = HttpRateLimiter::new(cfg);

        let t0 = Instant::now();
        let _ = lim.acquire_wait("x", false, t0).await;
        let _ = lim.acquire_wait("x", true, t0).await;

        let _ = lim.acquire_wait("x", true, t0).await;
        let _ = lim.acquire_wait("x", false, t0).await;
    }
}