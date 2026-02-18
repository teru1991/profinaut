//! D1 — Mongo bulk-insert sink.
//!
//! Design: `MongoTarget` trait provides the insert abstraction so unit tests
//! can use a `FakeMongoTarget` without a live server.  A real implementation
//! wrapping `mongodb::Collection<bson::Document>::insert_many` can be wired
//! in via the `real-mongo` feature (future work; marked NOT VERIFIED).
//!
//! State machine:
//!   OK ──(batch fail)──► MongoUnavailable
//!   MongoUnavailable ──(N consecutive batch failures)──► Degraded
//!   MongoUnavailable | Degraded ──(batch success)──► OK

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use super::envelope::Envelope;
use super::metrics::PersistenceMetrics;
use super::sink::{Sink, SinkError, SinkState};

// ---------------------------------------------------------------------------
// SinkState encoding
// ---------------------------------------------------------------------------

const STATE_OK: u8 = 0;
const STATE_UNAVAILABLE: u8 = 1;
const STATE_DEGRADED: u8 = 2;

fn decode_state(v: u8) -> SinkState {
    match v {
        STATE_OK => SinkState::Ok,
        STATE_UNAVAILABLE => SinkState::MongoUnavailable,
        _ => SinkState::Degraded,
    }
}

// ---------------------------------------------------------------------------
// MongoTarget trait
// ---------------------------------------------------------------------------

/// Abstraction over a Mongo collection's `insert_many` operation.
///
/// The real implementation (when `mongodb` crate is available) would be:
/// ```rust,ignore
/// #[async_trait]
/// impl MongoTarget for mongodb::Collection<bson::Document> {
///     async fn insert_many_envelopes(&self, envelopes: &[Envelope]) -> Result<(), String> {
///         let docs: Vec<bson::Document> = envelopes.iter()
///             .map(|e| bson::to_document(e).map_err(|e| e.to_string()))
///             .collect::<Result<_, _>>()?;
///         self.insert_many(docs, None).await
///             .map(|_| ())
///             .map_err(|e| e.to_string())
///     }
/// }
/// ```
#[async_trait]
pub trait MongoTarget: Send + Sync {
    /// Insert a batch of envelopes.  Returns `Ok(())` on success, or an error
    /// message string on failure (transient or permanent).
    async fn insert_many_envelopes(&self, envelopes: &[Envelope]) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// MongoSink
// ---------------------------------------------------------------------------

/// Configuration for `MongoSink`.
#[derive(Debug, Clone)]
pub struct MongoSinkConfig {
    /// Maximum number of retry attempts per batch (excluding the initial try).
    pub max_retries: u32,
    /// Base delay in milliseconds for exponential backoff.
    pub retry_base_ms: u64,
    /// Number of consecutive batch failures before transitioning to Degraded.
    pub consecutive_failures_for_degraded: u32,
}

impl Default for MongoSinkConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_base_ms: 100,
            consecutive_failures_for_degraded: 3,
        }
    }
}

/// Mongo bulk-insert sink.
///
/// Wraps a [`MongoTarget`] with bounded retry + exponential backoff and
/// tracks state transitions (OK → MongoUnavailable → Degraded).
pub struct MongoSink {
    target: Arc<dyn MongoTarget>,
    config: MongoSinkConfig,
    state: Arc<AtomicU8>,
    consecutive_failures: Arc<AtomicU32>,
    metrics: Arc<PersistenceMetrics>,
}

impl MongoSink {
    /// Create a new `MongoSink`.
    pub fn new(
        target: Arc<dyn MongoTarget>,
        config: MongoSinkConfig,
        metrics: Arc<PersistenceMetrics>,
    ) -> Self {
        Self {
            target,
            config,
            state: Arc::new(AtomicU8::new(STATE_OK)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            metrics,
        }
    }

    /// Extract the first exchange label from a batch (for error metrics).
    /// 注: この関数は、バッチ内のすべてのエンベロープが同じ取引所に属していると仮定しています。
    /// バッチに複数の取引所からのメッセージが含まれている場合、エラーは最初のエンベロープの取引所のみに帰属されます。
    fn batch_exchange(batch: &[Envelope]) -> &str {
        batch.first().map(|e| e.exchange.as_str()).unwrap_or("unknown")
    }
}

#[async_trait]
impl Sink for MongoSink {
    async fn write_batch(&self, batch: Vec<Envelope>) -> Result<(), SinkError> {
        if batch.is_empty() {
            return Ok(());
        }

        let exchange = Self::batch_exchange(&batch).to_string();
        let start = Instant::now();

        let mut last_err = String::new();

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                // Exponential backoff: base_ms * 2^(attempt-1)
                let delay_ms = self.config.retry_base_ms * (1u64 << (attempt - 1).min(6));
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }

            match self.target.insert_many_envelopes(&batch).await {
                Ok(()) => {
                    let latency_ms = start.elapsed().as_millis() as u64;
                    self.metrics.record_write_batch_latency(latency_ms);
                    self.state.store(STATE_OK, Ordering::Release);
                    self.consecutive_failures.store(0, Ordering::Release);
                    return Ok(());
                }
                Err(e) => {
                    last_err = e;
                }
            }
        }

        // All retries exhausted.
        let retries = self.config.max_retries;
        self.metrics.increment_ingest_errors(&exchange);

        let failures = self.consecutive_failures.fetch_add(1, Ordering::AcqRel) + 1;
        if failures >= self.config.consecutive_failures_for_degraded {
            self.state.store(STATE_DEGRADED, Ordering::Release);
        } else {
            self.state.store(STATE_UNAVAILABLE, Ordering::Release);
        }

        Err(SinkError::MongoUnavailable {
            retries,
            msg: last_err,
        })
    }

    fn state(&self) -> SinkState {
        decode_state(self.state.load(Ordering::Acquire))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::AtomicU32;
    use std::sync::Mutex;

    // ------------------------------------------------------------------
    // FakeMongoTarget — configurable fail/succeed behaviour
    // ------------------------------------------------------------------

    /// A test-only `MongoTarget` that fails a configurable number of times
    /// before succeeding.
    pub struct FakeMongoTarget {
        /// Number of remaining failures before returning Ok.
        pub remaining_failures: Arc<AtomicU32>,
        pub calls: Arc<AtomicU32>,
        pub inserted: Arc<Mutex<Vec<Vec<Envelope>>>>,
    }

    impl FakeMongoTarget {
        pub fn new(fail_times: u32) -> Arc<Self> {
            Arc::new(Self {
                remaining_failures: Arc::new(AtomicU32::new(fail_times)),
                calls: Arc::new(AtomicU32::new(0)),
                inserted: Arc::new(Mutex::new(Vec::new())),
            })
        }
    }

    #[async_trait]
    impl MongoTarget for FakeMongoTarget {
        async fn insert_many_envelopes(&self, envelopes: &[Envelope]) -> Result<(), String> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let prev = self.remaining_failures.load(Ordering::Acquire);
            if prev > 0 {
                self.remaining_failures.fetch_sub(1, Ordering::Release);
                return Err(format!("simulated failure ({prev} remaining)"));
            }
            self.inserted
                .lock()
                .unwrap()
                .push(envelopes.to_vec());
            Ok(())
        }
    }

    pub fn make_envelope(exchange: &str, channel: &str) -> Envelope {
        Envelope {
            message_id: Some("test-id".to_string()),
            sequence: Some(1),
            exchange: exchange.to_string(),
            channel: channel.to_string(),
            symbol: "BTC/USDT".to_string(),
            server_time_ms: None,
            received_at_ms: 0,
            payload: json!({}),
        }
    }

    #[tokio::test]
    async fn success_on_first_try_sets_state_ok() {
        let fake = FakeMongoTarget::new(0);
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(
            fake.clone(),
            MongoSinkConfig {
                max_retries: 3,
                retry_base_ms: 1,
                consecutive_failures_for_degraded: 3,
            },
            metrics.clone(),
        );

        let batch = vec![make_envelope("binance", "trades")];
        sink.write_batch(batch).await.unwrap();

        assert_eq!(sink.state(), SinkState::Ok);
        assert_eq!(fake.calls.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.ingest_errors_total.total(), 0);
    }

    #[tokio::test]
    async fn retry_succeeds_on_second_attempt() {
        let fake = FakeMongoTarget::new(1); // fail once, then succeed
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(
            fake.clone(),
            MongoSinkConfig {
                max_retries: 3,
                retry_base_ms: 1,
                consecutive_failures_for_degraded: 3,
            },
            metrics.clone(),
        );

        let batch = vec![make_envelope("kraken", "trades")];
        sink.write_batch(batch).await.unwrap();

        assert_eq!(sink.state(), SinkState::Ok);
        assert_eq!(fake.calls.load(Ordering::Relaxed), 2); // 1 fail + 1 success
        assert_eq!(metrics.ingest_errors_total.total(), 0);
    }

    #[tokio::test]
    async fn exhausted_retries_transitions_to_unavailable() {
        let fake = FakeMongoTarget::new(100); // always fail
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(
            fake.clone(),
            MongoSinkConfig {
                max_retries: 2,
                retry_base_ms: 1,
                consecutive_failures_for_degraded: 5,
            },
            metrics.clone(),
        );

        let batch = vec![make_envelope("binance", "trades")];
        assert!(sink.write_batch(batch).await.is_err());
        assert_eq!(sink.state(), SinkState::MongoUnavailable);
        assert_eq!(metrics.ingest_errors_total.get("binance"), 1);
    }

    #[tokio::test]
    async fn consecutive_failures_transition_to_degraded() {
        let fake = FakeMongoTarget::new(100); // always fail
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(
            fake.clone(),
            MongoSinkConfig {
                max_retries: 0, // no retries, fail fast
                retry_base_ms: 1,
                consecutive_failures_for_degraded: 3,
            },
            metrics.clone(),
        );

        let batch = || vec![make_envelope("binance", "trades")];

        // 1st failure → Unavailable
        let _ = sink.write_batch(batch()).await;
        assert_eq!(sink.state(), SinkState::MongoUnavailable);

        // 2nd failure → still Unavailable (2 < 3)
        let _ = sink.write_batch(batch()).await;
        assert_eq!(sink.state(), SinkState::MongoUnavailable);

        // 3rd failure → Degraded (3 >= 3)
        let _ = sink.write_batch(batch()).await;
        assert_eq!(sink.state(), SinkState::Degraded);
    }

    #[tokio::test]
    async fn recovery_after_degraded_resets_to_ok() {
        let fake = FakeMongoTarget::new(3); // fail 3 times, then succeed
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(
            fake.clone(),
            MongoSinkConfig {
                max_retries: 0,
                retry_base_ms: 1,
                consecutive_failures_for_degraded: 3,
            },
            metrics.clone(),
        );

        let batch = || vec![make_envelope("binance", "trades")];
        for _ in 0..3 {
            let _ = sink.write_batch(batch()).await;
        }
        assert_eq!(sink.state(), SinkState::Degraded);

        // Now succeeds → back to OK
        sink.write_batch(batch()).await.unwrap();
        assert_eq!(sink.state(), SinkState::Ok);
    }

    #[tokio::test]
    async fn empty_batch_is_noop() {
        let fake = FakeMongoTarget::new(100);
        let metrics = PersistenceMetrics::new();
        let sink = MongoSink::new(fake.clone(), MongoSinkConfig::default(), metrics);
        sink.write_batch(vec![]).await.unwrap();
        assert_eq!(fake.calls.load(Ordering::Relaxed), 0);
        assert_eq!(sink.state(), SinkState::Ok);
    }
}
