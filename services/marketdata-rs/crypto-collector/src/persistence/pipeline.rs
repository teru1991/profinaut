//! D5 — PipelineSink: integrates MongoSink + DurableSpool + DedupWindow.
//!
//! ## Fallback chain
//!
//! ```text
//! write_batch(batch)
//!   │
//!   ├─ [dedup enabled] → filter batch → drop duplicates
//!   │
//!   ├─ try MongoSink.write_batch(batch)
//!   │     └─ Ok  →  return Ok
//!   │
//!   └─ Err(MongoUnavailable) and spool enabled
//!         ├─ DurableSpool.append_batch(batch) → Ok  →  return Ok
//!         └─ spool full: on_full policy already enforced inside spool
//!               (DropAll / DropTickerDepthKeepTrade → Ok with drops)
//!               (Block → waits inside spool until space)
//! ```
//!
//! ## Stability guarantee
//!
//! `PipelineSink` implements the `Sink` trait, which is the stable interface
//! for Tasks E and F.  The trait signature must not change.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::watch;

use super::dedup::{DedupConfig, DedupWindow};
use super::envelope::Envelope;
use super::metrics::PersistenceMetrics;
use super::mongo::{MongoSink, MongoSinkConfig, MongoTarget};
use super::replay::{ReplayConfig, ReplayWorker};
use super::sink::{Sink, SinkError, SinkState};
use super::spool::{DurableSpool, SpoolConfig};

// ---------------------------------------------------------------------------
// PipelineConfig
// ---------------------------------------------------------------------------

/// Full configuration for the `PipelineSink`.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub mongo: MongoSinkConfig,
    pub spool: Option<SpoolConfig>,  // None → spool disabled
    pub dedup: Option<DedupConfig>,  // None → dedup disabled
    pub replay: ReplayConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            mongo: MongoSinkConfig::default(),
            spool: None,
            dedup: None,
            replay: ReplayConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// PipelineSink
// ---------------------------------------------------------------------------

/// The top-level sink that wires Mongo + spool + dedup + replay together.
pub struct PipelineSink {
    mongo: Arc<MongoSink>,
    spool: Option<Arc<DurableSpool>>,
    dedup: Option<Arc<DedupWindow>>,
    metrics: Arc<PersistenceMetrics>,
}

impl PipelineSink {
    /// Async builder: opens the spool (creating directories, recovering from
    /// partial writes) and spawns the replay worker.
    pub async fn build(
        mongo_target: Arc<dyn MongoTarget>,
        config: PipelineConfig,
        metrics: Arc<PersistenceMetrics>,
        shutdown_rx: Option<watch::Receiver<bool>>,
    ) -> Result<Self, SinkError> {
        let mongo = Arc::new(MongoSink::new(
            mongo_target.clone(),
            config.mongo.clone(),
            metrics.clone(),
        ));

        let spool = if let Some(sc) = config.spool.clone() {
            Some(DurableSpool::open(sc, metrics.clone()).await?)
        } else {
            None
        };

        let dedup = config.dedup.clone().map(|dc| DedupWindow::new(dc, metrics.clone()));

        // Spawn replay worker if spool is enabled.
        if let (Some(ref s), Some(rx)) = (&spool, shutdown_rx) {
            let worker = ReplayWorker::new(
                s.clone(),
                mongo_target,
                config.replay.clone(),
                metrics.clone(),
            );
            worker.spawn(rx);
        }

        Ok(Self { mongo, spool, dedup, metrics })
    }

    /// Access shared metrics for observability.
    pub fn metrics(&self) -> &Arc<PersistenceMetrics> {
        &self.metrics
    }
}

#[async_trait]
impl Sink for PipelineSink {
    async fn write_batch(&self, batch: Vec<Envelope>) -> Result<(), SinkError> {
        if batch.is_empty() {
            return Ok(());
        }

        // Step 1: dedup filter.
        let batch = if let Some(ref dw) = self.dedup {
            dw.filter(batch)
        } else {
            batch
        };

        if batch.is_empty() {
            return Ok(()); // all were duplicates
        }

        // Step 2: try Mongo.
        match self.mongo.write_batch(batch.clone()).await {
            Ok(()) => return Ok(()),
            Err(SinkError::MongoUnavailable { .. }) => {
                // fall through to spool
            }
            Err(e) => return Err(e),
        }

        // Step 3: spool fallback.
        if let Some(ref spool) = self.spool {
            spool.append_batch(batch).await?;
            return Ok(());
        }

        // No spool — propagate Mongo unavailability.
        Err(SinkError::MongoUnavailable {
            retries: 0,
            msg: "Mongo unavailable and spool is disabled".to_string(),
        })
    }

    fn state(&self) -> SinkState {
        self.mongo.state()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{
        metrics::PersistenceMetrics,
        mongo::tests::{FakeMongoTarget, make_envelope},
        spool::OnFullPolicy,
    };
    use std::sync::atomic::Ordering;
    use tempfile::TempDir;

    // Helper: PipelineSink with fake Mongo and optional spool.
    async fn make_pipeline(
        fail_times: u32,
        tmp: &std::path::Path,
        spool_enabled: bool,
    ) -> (PipelineSink, Arc<FakeMongoTarget>, Arc<PersistenceMetrics>) {
        let fake = FakeMongoTarget::new(fail_times);
        let metrics = PersistenceMetrics::new();

        let spool_cfg = if spool_enabled {
            Some(SpoolConfig {
                dir: tmp.to_path_buf(),
                max_segment_bytes: 1024 * 1024,
                max_total_bytes: 10 * 1024 * 1024,
                on_full: OnFullPolicy::DropAll,
            })
        } else {
            None
        };

        let config = PipelineConfig {
            mongo: MongoSinkConfig { max_retries: 0, retry_base_ms: 1, consecutive_failures_for_degraded: 3 },
            spool: spool_cfg,
            dedup: None,
            replay: ReplayConfig { batch_size: 10, rate_limit_ms: 0, poll_interval_ms: 100 },
        };

        let sink = PipelineSink::build(fake.clone(), config, metrics.clone(), None)
            .await
            .unwrap();

        (sink, fake, metrics)
    }

    // -----------------------------------------------------------------------
    // D6 fake integration test: spool grows then drains, metrics change
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn fake_integration_spool_grows_then_drains() {
        let tmp = TempDir::new().unwrap();
        let fake = FakeMongoTarget::new(3); // fail 3 times, then succeed
        let metrics = PersistenceMetrics::new();

        let config = PipelineConfig {
            mongo: MongoSinkConfig { max_retries: 0, retry_base_ms: 1, consecutive_failures_for_degraded: 5 },
            spool: Some(SpoolConfig {
                dir: tmp.path().to_path_buf(),
                // Small segment cap forces rotation after each envelope write,
                // so each write produces a complete segment available for replay.
                max_segment_bytes: 10,
                max_total_bytes: 10 * 1024 * 1024,
                on_full: OnFullPolicy::DropAll,
            }),
            dedup: None,
            replay: ReplayConfig::default(),
        };

        let sink = PipelineSink::build(fake.clone(), config, metrics.clone(), None)
            .await
            .unwrap();

        // --- Phase 1: Mongo fails → envelopes spooled ---
        for _ in 0..3 {
            let batch = vec![make_envelope("binance", "trades")];
            sink.write_batch(batch).await.unwrap(); // should not error; spool accepts it
        }

        let spool_bytes_after_phase1 = metrics.spool_bytes();
        assert!(spool_bytes_after_phase1 > 0, "spool_bytes should be > 0 after phase 1; got {spool_bytes_after_phase1}");

        // --- Phase 2: Mongo recovers → direct writes succeed ---
        let batch = vec![make_envelope("binance", "trades")];
        sink.write_batch(batch).await.unwrap();
        assert_eq!(sink.state(), SinkState::Ok);

        // --- Phase 3: Drain spool via replay worker ---
        let spool = sink.spool.as_ref().expect("spool should be present");
        let replay_worker = ReplayWorker::new(
            spool.clone(),
            fake.clone(),
            ReplayConfig { batch_size: 10, rate_limit_ms: 0, poll_interval_ms: 100 },
            metrics.clone(),
        );

        // Replay all complete segments.
        let complete_segs = spool.complete_segments().await.unwrap();
        for _ in &complete_segs {
            let _ = replay_worker.replay_oldest_segment().await;
        }

        let spool_replay = metrics.spool_replay_total();
        assert!(spool_replay > 0, "spool_replay_total should be > 0 after drain; got {spool_replay}");

        let ingest_errors = metrics.ingest_errors_total.total();
        assert!(ingest_errors >= 3, "expected ≥3 ingest errors, got {ingest_errors}");
    }

    #[tokio::test]
    async fn mongo_success_path_no_spool() {
        let tmp = TempDir::new().unwrap();
        let (sink, fake, metrics) = make_pipeline(0, tmp.path(), false).await;

        let batch = vec![make_envelope("binance", "trades")];
        sink.write_batch(batch).await.unwrap();

        assert_eq!(fake.calls.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.ingest_errors_total.total(), 0);
        assert_eq!(sink.state(), SinkState::Ok);
    }

    #[tokio::test]
    async fn mongo_unavailable_without_spool_returns_error() {
        let _tmp = TempDir::new().unwrap();
        let (sink, _, _) = make_pipeline(100, _tmp.path(), false).await;

        let batch = vec![make_envelope("binance", "trades")];
        let result = sink.write_batch(batch).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mongo_unavailable_with_spool_spools_ok() {
        let tmp = TempDir::new().unwrap();
        let (sink, _, metrics) = make_pipeline(100, tmp.path(), true).await;

        let batch = vec![make_envelope("kraken", "orderbook")];
        sink.write_batch(batch).await.unwrap();

        let spool_bytes = metrics.spool_bytes();
        assert!(spool_bytes > 0, "expected spool to have data, got {spool_bytes}");
    }

    #[tokio::test]
    async fn dedup_filter_drops_duplicates_in_pipeline() {
        let tmp = TempDir::new().unwrap();
        let fake = FakeMongoTarget::new(0);
        let metrics = PersistenceMetrics::new();

        let config = PipelineConfig {
            mongo: MongoSinkConfig { max_retries: 0, retry_base_ms: 1, consecutive_failures_for_degraded: 3 },
            spool: None,
            dedup: Some(DedupConfig { window_seconds: 300, max_keys: 1000 }),
            replay: ReplayConfig::default(),
        };

        let sink = PipelineSink::build(fake.clone(), config, metrics.clone(), None)
            .await
            .unwrap();

        let env = make_envelope("binance", "trades");
        sink.write_batch(vec![env.clone()]).await.unwrap();
        sink.write_batch(vec![env]).await.unwrap(); // duplicate

        assert_eq!(metrics.dedup_dropped_total.get("binance", "trades"), 1);
        // Mongo should have been called once (first), not twice.
        assert_eq!(fake.calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn empty_batch_is_noop_pipeline() {
        let tmp = TempDir::new().unwrap();
        let (sink, fake, _) = make_pipeline(0, tmp.path(), false).await;
        sink.write_batch(vec![]).await.unwrap();
        assert_eq!(fake.calls.load(Ordering::Relaxed), 0);
    }
}
