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

    pub inbound_frames: AtomicU64,
    pub inbound_bytes: AtomicU64,
    pub outbound_frames: AtomicU64,
    pub outbound_bytes: AtomicU64,
    pub decode_error: AtomicU64,
    pub last_inbound_unix_ms: AtomicI64,
    pub rl_wait_ms_total: AtomicU64,
    pub wal_write_latency_ms_last: AtomicI64,
    pub wal_write_latency_ms_max: AtomicI64,
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
            inbound_frames: AtomicU64::new(0),
            inbound_bytes: AtomicU64::new(0),
            outbound_frames: AtomicU64::new(0),
            outbound_bytes: AtomicU64::new(0),
            decode_error: AtomicU64::new(0),
            last_inbound_unix_ms: AtomicI64::new(0),
            rl_wait_ms_total: AtomicU64::new(0),
            wal_write_latency_ms_last: AtomicI64::new(-1),
            wal_write_latency_ms_max: AtomicI64::new(-1),
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

    #[inline]
    pub fn on_inbound(&self, bytes: usize, now_unix_ms: i64) {
        self.inbound_frames.fetch_add(1, Ordering::Relaxed);
        self.inbound_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
        self.last_inbound_unix_ms
            .store(now_unix_ms.max(0), Ordering::Relaxed);
        self.last_inbound_age_ms.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn on_outbound(&self, bytes: usize) {
        self.outbound_frames.fetch_add(1, Ordering::Relaxed);
        self.outbound_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    #[inline]
    pub fn on_decode_error(&self) {
        self.decode_error.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_outq_len(&self, v: i64) {
        self.outq_len.store(v, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_wal_queue_len(&self, v: i64) {
        self.wal_queue_len.store(v, Ordering::Relaxed);
    }

    #[inline]
    pub fn observe_wal_latency_ms(&self, ms: i64) {
        self.wal_write_latency_ms_last.store(ms, Ordering::Relaxed);
        let mut prev = self.wal_write_latency_ms_max.load(Ordering::Relaxed);
        while ms > prev {
            match self.wal_write_latency_ms_max.compare_exchange_weak(
                prev,
                ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => prev = p,
            }
        }
    }

    #[inline]
    pub fn observe_rl_wait_ms(&self, wait_ms: u64) {
        self.rl_wait_ms_total.fetch_add(wait_ms, Ordering::Relaxed);
    }

    #[inline]
    pub fn refresh_last_inbound_age_ms(&self, now_unix_ms: i64) {
        let last = self.last_inbound_unix_ms.load(Ordering::Relaxed);
        if last <= 0 {
            self.last_inbound_age_ms.store(-1, Ordering::Relaxed);
        } else {
            self.last_inbound_age_ms
                .store((now_unix_ms - last).max(0), Ordering::Relaxed);
        }
    }
}
