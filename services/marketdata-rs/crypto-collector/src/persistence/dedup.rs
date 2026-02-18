//! D4 — Dedup window (optional toggle).
//!
//! ## Key rules (in priority order)
//! 1. `message_id` → `"mid:<id>"`
//! 2. `sequence`   → `"seq:<exchange>:<channel>:<seq>"`
//! 3. payload hash → `"hash:<16 hex chars>"`
//!
//! ## Eviction
//!
//! Entries older than `window_seconds` are evicted lazily on each `filter()`
//! call.  When `len > max_keys` the oldest entries are also evicted to keep
//! the window bounded (no memory leak).
//!
//! ## Thread safety
//!
//! `DedupWindow` is `Send + Sync`; all internal state is guarded by a single
//! `Mutex`.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::envelope::Envelope;
use super::metrics::PersistenceMetrics;

// ---------------------------------------------------------------------------
// DedupConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DedupConfig {
    /// Seconds to keep a key before eviction.
    pub window_seconds: u64,
    /// Hard cap on number of live keys (evict oldest when exceeded).
    pub max_keys: usize,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            window_seconds: 300,
            max_keys: 100_000,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

struct Inner {
    /// (dedup_key, insertion_instant) in insertion order (oldest at front).
    queue: VecDeque<(String, Instant)>,
    /// O(1) membership test.
    seen: HashSet<String>,
}

impl Inner {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            seen: HashSet::new(),
        }
    }

    /// Evict entries older than `window` and trim to `max_keys`.
    fn evict(&mut self, window: Duration, max_keys: usize) {
        let now = Instant::now();

        // Evict by time (oldest first).
        while let Some((key, ts)) = self.queue.front() {
            if now.duration_since(*ts) > window {
                let key = key.clone();
                self.queue.pop_front();
                self.seen.remove(&key);
            } else {
                break;
            }
        }

        // Evict by max_keys cap (oldest first).
        // Trim to max_keys - 1 to make room for the next insertion so that
        // after the caller inserts a new entry len stays <= max_keys.
        while self.queue.len() >= max_keys {
            if let Some((key, _)) = self.queue.pop_front() {
                self.seen.remove(&key);
            }
        }
    }

    /// Check if `key` is a duplicate.  If not, record it.
    /// Returns `true` if `key` was already seen (is a duplicate).
    fn check_and_mark(&mut self, key: String, window: Duration, max_keys: usize) -> bool {
        self.evict(window, max_keys);

        if self.seen.contains(&key) {
            return true; // duplicate
        }

        self.seen.insert(key.clone());
        self.queue.push_back((key, Instant::now()));
        false
    }
}

// ---------------------------------------------------------------------------
// DedupWindow
// ---------------------------------------------------------------------------

/// Bounded, time-windowed deduplication filter.
pub struct DedupWindow {
    config: DedupConfig,
    inner: Mutex<Inner>,
    metrics: Arc<PersistenceMetrics>,
}

impl DedupWindow {
    pub fn new(config: DedupConfig, metrics: Arc<PersistenceMetrics>) -> Arc<Self> {
        Arc::new(Self {
            config,
            inner: Mutex::new(Inner::new()),
            metrics,
        })
    }

    /// Filter `batch` and return only non-duplicate envelopes.
    /// Duplicate envelopes increment `dedup_dropped_total{exchange,channel}`.
    pub fn filter(&self, batch: Vec<Envelope>) -> Vec<Envelope> {
        let window = Duration::from_secs(self.config.window_seconds);
        let max_keys = self.config.max_keys;
        let mut guard = self.inner.lock().unwrap();

        batch
            .into_iter()
            .filter(|env| {
                let key = env.dedup_key();
                if guard.check_and_mark(key, window, max_keys) {
                    self.metrics
                        .increment_dedup_dropped(&env.exchange, &env.channel);
                    false
                } else {
                    true
                }
            })
            .collect()
    }

    /// Number of currently live keys (for observability/tests).
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_env(message_id: Option<&str>, channel: &str) -> Envelope {
        Envelope {
            message_id: message_id.map(str::to_string),
            sequence: None,
            exchange: "binance".to_string(),
            channel: channel.to_string(),
            symbol: "BTC/USDT".to_string(),
            server_time_ms: None,
            received_at_ms: 0,
            payload: json!({}),
        }
    }

    #[test]
    fn passes_unique_messages() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(DedupConfig::default(), metrics.clone());

        let batch = vec![
            make_env(Some("a"), "trades"),
            make_env(Some("b"), "trades"),
            make_env(Some("c"), "trades"),
        ];
        let out = dw.filter(batch);
        assert_eq!(out.len(), 3);
        assert_eq!(metrics.dedup_dropped_total.total(), 0);
    }

    #[test]
    fn drops_duplicate_message_id() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(DedupConfig::default(), metrics.clone());

        let e = make_env(Some("dup"), "trades");
        let out = dw.filter(vec![e.clone(), e]);
        assert_eq!(out.len(), 1);
        assert_eq!(metrics.dedup_dropped_total.get("binance", "trades"), 1);
    }

    #[test]
    fn drops_duplicate_across_batches() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(DedupConfig::default(), metrics.clone());

        let e = make_env(Some("x"), "trades");
        dw.filter(vec![e.clone()]);
        let out = dw.filter(vec![e]);
        assert_eq!(out.len(), 0);
        assert_eq!(metrics.dedup_dropped_total.total(), 1);
    }

    #[test]
    fn eviction_by_max_keys() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(
            DedupConfig { window_seconds: 3600, max_keys: 3 },
            metrics.clone(),
        );

        // Insert 4 unique keys → oldest should be evicted to keep len ≤ 3.
        for i in 0..4u32 {
            dw.filter(vec![make_env(Some(&format!("id-{i}")), "trades")]);
        }
        assert!(dw.len() <= 3, "expected len ≤ 3, got {}", dw.len());
    }

    #[test]
    fn eviction_by_time() {
        // We can't fast-forward Instant, so we use a 0-second window.
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(
            DedupConfig { window_seconds: 0, max_keys: 100_000 },
            metrics.clone(),
        );

        let e = make_env(Some("y"), "trades");
        // First insert.
        dw.filter(vec![e.clone()]);
        // With window=0 every entry is immediately expired on next eviction.
        // The second call should evict "y" and then re-insert it → not a dup.
        let out = dw.filter(vec![e]);
        assert_eq!(out.len(), 1, "should not be considered dup after expiry");
    }

    #[test]
    fn seq_key_dedup() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(DedupConfig::default(), metrics.clone());

        let mut e = make_env(None, "trades");
        e.sequence = Some(99);
        let out = dw.filter(vec![e.clone(), e]);
        assert_eq!(out.len(), 1);
        assert_eq!(metrics.dedup_dropped_total.total(), 1);
    }

    #[test]
    fn labeled_metric_tracks_exchange_channel() {
        let metrics = PersistenceMetrics::new();
        let dw = DedupWindow::new(DedupConfig::default(), metrics.clone());

        let mut ob = make_env(Some("ob-1"), "orderbook");
        ob.exchange = "kraken".to_string();
        dw.filter(vec![ob.clone(), ob]);

        assert_eq!(metrics.dedup_dropped_total.get("kraken", "orderbook"), 1);
        assert_eq!(metrics.dedup_dropped_total.get("binance", "trades"), 0);
    }
}
