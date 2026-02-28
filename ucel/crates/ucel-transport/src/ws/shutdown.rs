//! Graceful shutdown helpers.
//!
//! Target sequence (spec-fixed):
//! 1) close (request close)
//! 2) flush (drain outbound queue + WAL queue)
//! 3) requeue (active/inflight -> pending)
//! 4) join (writer + wal tasks)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use ucel_subscription_store::SubscriptionStore;

use super::overflow::{DropMode, OverflowPolicy};
use super::priority::{PriorityQueue, QueuedOutbound};

#[derive(Clone, Debug)]
pub struct ShutdownToken {
    pub flag: Arc<AtomicBool>,
}

impl ShutdownToken {
    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct GracefulShutdownConfig {
    /// Max time to wait for outbound queue drain.
    pub drain_timeout: Duration,
    /// Max time to wait for joining tasks.
    pub join_timeout: Duration,
}

impl Default for GracefulShutdownConfig {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_secs(5),
            join_timeout: Duration::from_secs(5),
        }
    }
}

fn now_unix_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Perform graceful shutdown.
///
/// - Requests a close via the outbound queue.
/// - Prevents new non-control frames.
/// - Waits for drain.
/// - Requeues subscriptions.
/// - Joins writer + wal tasks.
pub async fn graceful_shutdown_ws(
    cfg: GracefulShutdownConfig,
    exchange_id: &str,
    conn_id: &str,
    store: &SubscriptionStore,
    outq: &Arc<PriorityQueue>,
    shutdown: &ShutdownToken,
    mut writer: JoinHandle<()>,
    mut wal_writer: JoinHandle<()>,
) -> Result<(), String> {
    // 1) close (request close)
    shutdown.trigger();
    outq.begin_closing();

    // Ensure close request isn't dropped even when full.
    let close_policy = OverflowPolicy::Drop {
        mode: DropMode::DropOldestLowPriority,
    };

    let _ = outq
        .push(exchange_id, conn_id, QueuedOutbound::close_request(), &close_policy, 0)
        .await;

    // 2) flush (drain outbound queue)
    let drain_deadline = Instant::now() + cfg.drain_timeout;
    while Instant::now() < drain_deadline {
        if outq.is_empty().await {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    outq.close();

    // 3) requeue
    let _ = store.requeue_active_to_pending(exchange_id, conn_id, now_unix_i64());

    // 4) join
    let join_writer = tokio::time::timeout(cfg.join_timeout, &mut writer).await;
    if join_writer.is_err() {
        writer.abort();
    }

    let join_wal = tokio::time::timeout(cfg.join_timeout, &mut wal_writer).await;
    if join_wal.is_err() {
        wal_writer.abort();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_token_triggers() {
        let t = ShutdownToken {
            flag: Arc::new(AtomicBool::new(false)),
        };
        assert!(!t.is_triggered());
        t.trigger();
        assert!(t.is_triggered());
    }
}