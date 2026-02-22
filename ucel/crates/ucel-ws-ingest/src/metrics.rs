use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Minimal in-process metrics counters for ws-ingest operational observability.
#[derive(Debug, Default)]
pub struct IngestMetrics {
    pub ws_connected: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub subscribe_sent_total: AtomicU64,
    pub subscribe_fail_total: AtomicU64,
    pub active_subscriptions: AtomicU64,
    pub deadletter_total: AtomicU64,
    pub journal_appends_total: AtomicU64,
    pub journal_bytes_total: AtomicU64,
    pub stall_detected_total: AtomicU64,
}

impl IngestMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_connected(&self) {
        self.ws_connected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_reconnect(&self) {
        self.ws_reconnect_total.fetch_add(1, Ordering::Relaxed);
        // Atomically decrement ws_connected, saturating at 0.
        let _ = self.ws_connected.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |prev| if prev > 0 { Some(prev - 1) } else { None },
        );
    }

    pub fn record_subscribe_sent(&self) {
        self.subscribe_sent_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_subscribe_fail(&self) {
        self.subscribe_fail_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_active(&self) {
        self.active_subscriptions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_deadletter(&self) {
        self.deadletter_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_journal_append(&self, bytes: u64) {
        self.journal_appends_total.fetch_add(1, Ordering::Relaxed);
        self.journal_bytes_total.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_stall(&self) {
        self.stall_detected_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            ws_connected: self.ws_connected.load(Ordering::Relaxed),
            ws_reconnect_total: self.ws_reconnect_total.load(Ordering::Relaxed),
            subscribe_sent_total: self.subscribe_sent_total.load(Ordering::Relaxed),
            subscribe_fail_total: self.subscribe_fail_total.load(Ordering::Relaxed),
            active_subscriptions: self.active_subscriptions.load(Ordering::Relaxed),
            deadletter_total: self.deadletter_total.load(Ordering::Relaxed),
            journal_appends_total: self.journal_appends_total.load(Ordering::Relaxed),
            journal_bytes_total: self.journal_bytes_total.load(Ordering::Relaxed),
            stall_detected_total: self.stall_detected_total.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub ws_connected: u64,
    pub ws_reconnect_total: u64,
    pub subscribe_sent_total: u64,
    pub subscribe_fail_total: u64,
    pub active_subscriptions: u64,
    pub deadletter_total: u64,
    pub journal_appends_total: u64,
    pub journal_bytes_total: u64,
    pub stall_detected_total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_record_and_snapshot() {
        let m = IngestMetrics::new();
        m.record_connected();
        m.record_subscribe_sent();
        m.record_journal_append(128);
        let s = m.snapshot();
        assert_eq!(s.ws_connected, 1);
        assert_eq!(s.subscribe_sent_total, 1);
        assert_eq!(s.journal_bytes_total, 128);
    }
}
