use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct TransportMetrics {
    pub reconnect_attempts: AtomicU64,
    pub reconnect_success: AtomicU64,
    pub reconnect_failure: AtomicU64,
    pub breaker_open: AtomicU64,
    pub stale_requeued: AtomicU64,
    pub outq_dropped: AtomicU64,
    pub outq_spilled: AtomicU64,
    pub rl_penalty_applied: AtomicU64,
    pub rl_cooldown_set: AtomicU64,
    pub deadletter_count: AtomicU64,

    pub outq_len: AtomicI64,
    pub wal_queue_len: AtomicI64,
    pub last_inbound_age_ms: AtomicI64,
}

impl TransportMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            reconnect_attempts: AtomicU64::new(0),
            reconnect_success: AtomicU64::new(0),
            reconnect_failure: AtomicU64::new(0),
            breaker_open: AtomicU64::new(0),
            stale_requeued: AtomicU64::new(0),
            outq_dropped: AtomicU64::new(0),
            outq_spilled: AtomicU64::new(0),
            rl_penalty_applied: AtomicU64::new(0),
            rl_cooldown_set: AtomicU64::new(0),
            deadletter_count: AtomicU64::new(0),
            outq_len: AtomicI64::new(0),
            wal_queue_len: AtomicI64::new(0),
            last_inbound_age_ms: AtomicI64::new(-1),
        })
    }

    #[inline]
    pub fn inc(c: &AtomicU64) {
        c.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn add(c: &AtomicU64, v: u64) {
        c.fetch_add(v, Ordering::Relaxed);
    }

    #[inline]
    pub fn set(g: &AtomicI64, v: i64) {
        g.store(v, Ordering::Relaxed);
    }
}
