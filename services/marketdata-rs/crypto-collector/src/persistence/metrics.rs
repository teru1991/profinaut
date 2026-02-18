//! Persistence metrics — atomic counters and labeled counters.
//!
//! Intentionally dependency-free (no prometheus / metrics crate required).
//! In a production deployment these would be wired to a metrics registry;
//! here they are observable via test assertions and structured-log snapshots.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Simple labeled counter (exchange, channel) → u64
// ---------------------------------------------------------------------------

/// A two-label (exchange × channel) counter.
#[derive(Debug, Default)]
pub struct ExchangeChannelCounter {
    data: Mutex<HashMap<(String, String), u64>>,
}

impl ExchangeChannelCounter {
    pub fn increment(&self, exchange: &str, channel: &str) {
        let mut m = self.data.lock().unwrap();
        *m.entry((exchange.to_string(), channel.to_string()))
            .or_insert(0) += 1;
    }

    pub fn get(&self, exchange: &str, channel: &str) -> u64 {
        let m = self.data.lock().unwrap();
        *m.get(&(exchange.to_string(), channel.to_string()))
            .unwrap_or(&0)
    }

    pub fn total(&self) -> u64 {
        self.data.lock().unwrap().values().sum()
    }
}

// ---------------------------------------------------------------------------
// Single-label (exchange) counter
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ExchangeCounter {
    data: Mutex<HashMap<String, u64>>,
}

impl ExchangeCounter {
    pub fn increment(&self, exchange: &str) {
        let mut m = self.data.lock().unwrap();
        *m.entry(exchange.to_string()).or_insert(0) += 1;
    }

    pub fn get(&self, exchange: &str) -> u64 {
        let m = self.data.lock().unwrap();
        *m.get(exchange).unwrap_or(&0)
    }

    pub fn total(&self) -> u64 {
        self.data.lock().unwrap().values().sum()
    }
}

// ---------------------------------------------------------------------------
// PersistenceMetrics
// ---------------------------------------------------------------------------

/// All persistence metrics in one struct.
///
/// Shared via `Arc<PersistenceMetrics>` between the sink components.
#[derive(Debug, Default)]
pub struct PersistenceMetrics {
    // D1 -----------------------------------------------------------------------
    /// Last observed write-batch latency in milliseconds.
    pub write_batch_latency_ms: AtomicU64,
    /// ingest_errors_total{exchange}
    pub ingest_errors_total: ExchangeCounter,

    // D2 -----------------------------------------------------------------------
    /// spool_bytes (gauge — total bytes on disk across all segments)
    pub spool_bytes: AtomicI64,
    /// spool_segments (gauge)
    pub spool_segments: AtomicU64,
    /// spool_dropped_total{exchange,channel}
    pub spool_dropped_total: ExchangeChannelCounter,

    // D3 -----------------------------------------------------------------------
    /// spool_replay_total (cumulative envelopes successfully replayed)
    pub spool_replay_total: AtomicU64,

    // D4 -----------------------------------------------------------------------
    /// dedup_dropped_total{exchange,channel}
    pub dedup_dropped_total: ExchangeChannelCounter,
}

impl PersistenceMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    // D1 helpers ---------------------------------------------------------------

    pub fn record_write_batch_latency(&self, ms: u64) {
        self.write_batch_latency_ms.store(ms, Ordering::Relaxed);
    }

    pub fn increment_ingest_errors(&self, exchange: &str) {
        self.ingest_errors_total.increment(exchange);
    }

    // D2 helpers ---------------------------------------------------------------

    pub fn set_spool_bytes(&self, bytes: i64) {
        self.spool_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn add_spool_bytes(&self, delta: i64) {
        self.spool_bytes.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn spool_bytes(&self) -> i64 {
        self.spool_bytes.load(Ordering::Relaxed)
    }

    pub fn set_spool_segments(&self, count: u64) {
        self.spool_segments.store(count, Ordering::Relaxed);
    }

    pub fn spool_segments(&self) -> u64 {
        self.spool_segments.load(Ordering::Relaxed)
    }

    pub fn increment_spool_dropped(&self, exchange: &str, channel: &str) {
        self.spool_dropped_total.increment(exchange, channel);
    }

    // D3 helpers ---------------------------------------------------------------

    pub fn add_spool_replay(&self, count: u64) {
        self.spool_replay_total.fetch_add(count, Ordering::Relaxed);
    }

    pub fn spool_replay_total(&self) -> u64 {
        self.spool_replay_total.load(Ordering::Relaxed)
    }

    // D4 helpers ---------------------------------------------------------------

    pub fn increment_dedup_dropped(&self, exchange: &str, channel: &str) {
        self.dedup_dropped_total.increment(exchange, channel);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_channel_counter_increments_and_reads() {
        let c = ExchangeChannelCounter::default();
        c.increment("binance", "trades");
        c.increment("binance", "trades");
        c.increment("kraken", "orderbook");
        assert_eq!(c.get("binance", "trades"), 2);
        assert_eq!(c.get("kraken", "orderbook"), 1);
        assert_eq!(c.get("kraken", "trades"), 0);
        assert_eq!(c.total(), 3);
    }

    #[test]
    fn exchange_counter_increments_and_reads() {
        let c = ExchangeCounter::default();
        c.increment("binance");
        c.increment("binance");
        c.increment("kraken");
        assert_eq!(c.get("binance"), 2);
        assert_eq!(c.get("kraken"), 1);
        assert_eq!(c.get("unknown"), 0);
        assert_eq!(c.total(), 3);
    }

    #[test]
    fn persistence_metrics_arc_shared() {
        let m = PersistenceMetrics::new();
        m.record_write_batch_latency(42);
        assert_eq!(m.write_batch_latency_ms.load(std::sync::atomic::Ordering::Relaxed), 42);

        m.increment_ingest_errors("binance");
        assert_eq!(m.ingest_errors_total.get("binance"), 1);

        m.set_spool_bytes(1024);
        assert_eq!(m.spool_bytes(), 1024);
        m.add_spool_bytes(512);
        assert_eq!(m.spool_bytes(), 1536);

        m.set_spool_segments(3);
        assert_eq!(m.spool_segments(), 3);

        m.increment_spool_dropped("kraken", "trades");
        assert_eq!(m.spool_dropped_total.get("kraken", "trades"), 1);

        m.add_spool_replay(7);
        assert_eq!(m.spool_replay_total(), 7);

        m.increment_dedup_dropped("binance", "orderbook");
        assert_eq!(m.dedup_dropped_total.get("binance", "orderbook"), 1);
    }
}
