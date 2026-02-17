//! D3 — Replay worker: drains the durable spool into Mongo after recovery.
//!
//! ## Behaviour
//!
//! * Replays segments in ascending sequence-number order (oldest first).
//! * A segment is **deleted only after** a successful `insert_many` call.
//! * Rate-limited: `rate_limit_ms` milliseconds of sleep between successful
//!   batch replays to avoid overwhelming Mongo after a long outage.
//! * **Shutdown-safe**: monitors a `watch::Receiver<bool>` shutdown signal.
//!   On signal the worker drains the current batch (if in-flight) and exits
//!   cleanly — it does **not** delete a segment if the insert is still pending.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use super::envelope::Envelope;
use super::metrics::PersistenceMetrics;
use super::mongo::MongoTarget;
use super::spool::DurableSpool;

// ---------------------------------------------------------------------------
// ReplayConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// How many envelopes to replay per batch.
    pub batch_size: usize,
    /// Milliseconds to sleep between successful replay batches (rate limit).
    pub rate_limit_ms: u64,
    /// Milliseconds to sleep between idle polls (no segments available).
    pub poll_interval_ms: u64,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,
            rate_limit_ms: 200,
            poll_interval_ms: 5_000,
        }
    }
}

// ---------------------------------------------------------------------------
// ReplayWorker
// ---------------------------------------------------------------------------

/// Background task that replays the durable spool into a `MongoTarget`.
pub struct ReplayWorker {
    spool: Arc<DurableSpool>,
    target: Arc<dyn MongoTarget>,
    config: ReplayConfig,
    metrics: Arc<PersistenceMetrics>,
}

impl ReplayWorker {
    pub fn new(
        spool: Arc<DurableSpool>,
        target: Arc<dyn MongoTarget>,
        config: ReplayConfig,
        metrics: Arc<PersistenceMetrics>,
    ) -> Self {
        Self { spool, target, config, metrics }
    }

    /// Spawn the worker as a background Tokio task.
    ///
    /// The caller controls shutdown by sending `true` through `shutdown_tx`.
    /// The returned `JoinHandle` resolves when the task exits cleanly.
    pub fn spawn(
        self,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.run(&mut shutdown_rx).await;
        })
    }

    /// Main replay loop.  Exits when shutdown signal is received.
    pub async fn run(&self, shutdown_rx: &mut watch::Receiver<bool>) {
        loop {
            // Check shutdown before each iteration.
            if *shutdown_rx.borrow() {
                break;
            }

            match self.replay_oldest_segment().await {
                Ok(true) => {
                    // A segment was replayed; apply rate limit.
                    tokio::time::sleep(Duration::from_millis(self.config.rate_limit_ms)).await;
                }
                Ok(false) => {
                    // No segments ready; poll interval sleep.
                    tokio::select! {
                        () = tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)) => {}
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() { break; }
                        }
                    }
                }
                Err(e) => {
                    // Log and back off on error.
                    tracing::warn!(error = %e, "replay error; backing off");
                    tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                }
            }
        }
    }

    /// Replay the oldest complete segment.
    ///
    /// Returns `Ok(true)` if a segment was processed, `Ok(false)` if there
    /// were no complete segments to replay.
    pub async fn replay_oldest_segment(&self) -> Result<bool, String> {
        let mut seqs = self
            .spool
            .complete_segments()
            .await
            .map_err(|e| e.to_string())?;

        seqs.sort();
        let seq = match seqs.into_iter().next() {
            Some(s) => s,
            None => return Ok(false),
        };

        let envelopes: Vec<Envelope> = self
            .spool
            .read_segment(seq)
            .await
            .map_err(|e| e.to_string())?;

        if envelopes.is_empty() {
            // Empty/corrupt segment — delete it.
            self.spool
                .delete_segment(seq)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(true);
        }

        // Replay in chunks of `batch_size`.
        for chunk in envelopes.chunks(self.config.batch_size) {
            self.target
                .insert_many_envelopes(chunk)
                .await
                .map_err(|e| e)?;
            self.metrics.add_spool_replay(chunk.len() as u64);
        }

        // Delete segment ONLY after all chunks succeeded.
        self.spool
            .delete_segment(seq)
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
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
        spool::SpoolConfig,
    };
    use std::sync::atomic::Ordering;
    use tempfile::TempDir;

    async fn make_spool(dir: &std::path::Path) -> Arc<DurableSpool> {
        let config = SpoolConfig {
            dir: dir.to_path_buf(),
            max_segment_bytes: 10, // tiny → each write produces a new segment
            max_total_bytes: 100 * 1024 * 1024,
            on_full: crate::persistence::spool::OnFullPolicy::DropAll,
        };
        let metrics = PersistenceMetrics::new();
        DurableSpool::open(config, metrics).await.unwrap()
    }

    #[tokio::test]
    async fn replay_drains_segment_after_success() {
        let tmp = TempDir::new().unwrap();
        let metrics = PersistenceMetrics::new();
        let spool = make_spool(tmp.path()).await;

        // Write two envelopes (tiny segment → two segments after two writes).
        spool.append_batch(vec![make_envelope("binance", "trades")]).await.unwrap();
        spool.append_batch(vec![make_envelope("kraken", "trades")]).await.unwrap();

        let fake = FakeMongoTarget::new(0); // always succeed
        let worker = ReplayWorker::new(
            spool.clone(),
            fake.clone(),
            ReplayConfig { batch_size: 10, rate_limit_ms: 0, poll_interval_ms: 100 },
            metrics.clone(),
        );

        // Replay one segment.
        let replayed = worker.replay_oldest_segment().await.unwrap();
        assert!(replayed, "expected a segment to be replayed");
        assert!(fake.calls.load(Ordering::Relaxed) >= 1);
        assert!(metrics.spool_replay_total() >= 1);
    }

    #[tokio::test]
    async fn replay_does_not_delete_on_failure() {
        let tmp = TempDir::new().unwrap();
        let metrics = PersistenceMetrics::new();
        let spool = make_spool(tmp.path()).await;

        spool.append_batch(vec![make_envelope("binance", "trades")]).await.unwrap();
        // Force a rotation so we have at least one complete segment.
        spool.append_batch(vec![make_envelope("binance", "trades")]).await.unwrap();

        let complete_before = spool.complete_segments().await.unwrap().len();

        let fake = FakeMongoTarget::new(100); // always fail
        let worker = ReplayWorker::new(
            spool.clone(),
            fake.clone(),
            ReplayConfig::default(),
            metrics.clone(),
        );

        let result = worker.replay_oldest_segment().await;
        assert!(result.is_err(), "expected error on Mongo failure");

        let complete_after = spool.complete_segments().await.unwrap().len();
        assert_eq!(
            complete_before, complete_after,
            "segment should NOT be deleted after failed replay"
        );
    }

    #[tokio::test]
    async fn shutdown_stops_worker() {
        let tmp = TempDir::new().unwrap();
        let metrics = PersistenceMetrics::new();
        let spool = make_spool(tmp.path()).await;
        let fake = FakeMongoTarget::new(0);

        let worker = ReplayWorker::new(
            spool,
            fake,
            ReplayConfig { batch_size: 10, rate_limit_ms: 0, poll_interval_ms: 50 },
            metrics,
        );

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let handle = worker.spawn(shutdown_rx);

        // Give the worker a moment to start, then signal shutdown.
        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(true).unwrap();

        // Worker should exit within a reasonable time.
        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("worker did not exit within 2s")
            .unwrap();
    }
}
