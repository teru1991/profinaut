use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;

use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};

use ucel_transport::ws::overflow::{DropMode, OverflowPolicy};
use ucel_transport::ws::priority::{
    OutboundPriority, PriorityQueue, QueuedOutbound, WsOutboundFrame,
};
use ucel_transport::ws::shutdown::{graceful_shutdown_ws, GracefulShutdownConfig, ShutdownToken};

fn now_unix_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn now_unix_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn spawn_fake_writer(q: Arc<PriorityQueue>) -> JoinHandle<()> {
    tokio::spawn(async move {
        // Simulate a WS writer task:
        // - consumes outbound frames
        // - exits when CloseRequest is observed or queue is closed & drained.
        let mut saw_close = false;
        loop {
            match q.recv().await {
                None => break,
                Some(item) => {
                    if matches!(item.frame, WsOutboundFrame::CloseRequest) {
                        saw_close = true;
                        continue;
                    }
                    // emulate doing IO
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
            if saw_close && q.is_empty().await {
                break;
            }
        }
    })
}

async fn spawn_fake_wal_writer(shutdown: ShutdownToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        // In real code, WAL writer would flush buffers and stop.
        // Here we just wait until shutdown is triggered.
        while !shutdown.is_triggered() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

async fn wait_until_queue_empty(q: &PriorityQueue, timeout: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if q.is_empty().await {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn close_flush_requeue_join_is_enforced() {
    // Arrange
    let exchange_id = "bitbank";
    let conn_id = "conn-1";
    let now = now_unix_i64();

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    store
        .seed(
            &[
                SubscriptionRow {
                    key: "k1".into(),
                    exchange_id: exchange_id.into(),
                    op_id: "crypto.public.ticker".into(),
                    symbol: Some("BTC/JPY".into()),
                    params_json: "{}".into(),
                    assigned_conn: Some(conn_id.into()),
                },
                SubscriptionRow {
                    key: "k2".into(),
                    exchange_id: exchange_id.into(),
                    op_id: "crypto.private.orders".into(),
                    symbol: None,
                    params_json: "{}".into(),
                    assigned_conn: Some(conn_id.into()),
                },
            ],
            now,
        )
        .unwrap();

    // mark them active (so requeue must move them to pending)
    store.mark_active("k1", now).unwrap();
    store.mark_active("k2", now).unwrap();

    let q = PriorityQueue::new(8);
    let policy = OverflowPolicy::Drop {
        mode: DropMode::DropOldestLowPriority,
    };

    // push some frames that must be drained (flush)
    let ts = now_unix_u64();
    q.push(
        exchange_id,
        conn_id,
        QueuedOutbound {
            priority: OutboundPriority::Public,
            op_id: Some("crypto.public.ticker".into()),
            symbol: Some("BTC/JPY".into()),
            frame: WsOutboundFrame::Text("{\"type\":\"sub\"}".into()),
            meta: serde_json::json!({"t":"public-sub"}),
        },
        &policy,
        ts,
    )
    .await
    .unwrap();

    q.push(
        exchange_id,
        conn_id,
        QueuedOutbound {
            priority: OutboundPriority::Private,
            op_id: Some("crypto.private.orders".into()),
            symbol: None,
            frame: WsOutboundFrame::Text("{\"type\":\"auth\"}".into()),
            meta: serde_json::json!({"t":"private-auth"}),
        },
        &policy,
        ts,
    )
    .await
    .unwrap();

    let shutdown = ShutdownToken {
        flag: Arc::new(AtomicBool::new(false)),
    };

    let writer = spawn_fake_writer(q.clone()).await;
    let wal = spawn_fake_wal_writer(shutdown.clone()).await;

    // Act
    let cfg = GracefulShutdownConfig {
        drain_timeout: Duration::from_secs(2),
        join_timeout: Duration::from_secs(2),
    };

    graceful_shutdown_ws(
        cfg,
        exchange_id,
        conn_id,
        &store,
        &q,
        &shutdown,
        writer,
        wal,
    )
    .await
    .unwrap();

    // Assert: queue is closed/drained
    assert!(wait_until_queue_empty(&q, Duration::from_millis(200)).await);

    // Assert: active/inflight -> pending
    let s1 = store.state_of("k1").unwrap().unwrap();
    let s2 = store.state_of("k2").unwrap().unwrap();
    assert_eq!(s1, "pending");
    assert_eq!(s2, "pending");
}

#[tokio::test(flavor = "current_thread")]
async fn closing_rejects_non_control_frames_but_allows_close_request() {
    let exchange_id = "x";
    let conn_id = "c";
    let q = PriorityQueue::new(1);
    let policy = OverflowPolicy::drop_newest();
    let ts = 1u64;

    // Fill capacity with a public frame
    q.push(
        exchange_id,
        conn_id,
        QueuedOutbound {
            priority: OutboundPriority::Public,
            op_id: Some("crypto.public.ticker".into()),
            symbol: None,
            frame: WsOutboundFrame::Text("a".into()),
            meta: serde_json::json!({}),
        },
        &policy,
        ts,
    )
    .await
    .unwrap();

    // Begin closing => reject non-control
    q.begin_closing();

    let out = q
        .push(
            exchange_id,
            conn_id,
            QueuedOutbound {
                priority: OutboundPriority::Public,
                op_id: Some("crypto.public.book".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("b".into()),
                meta: serde_json::json!({}),
            },
            &policy,
            ts,
        )
        .await
        .unwrap();
    assert_eq!(out, ucel_transport::ws::priority::PushOutcome::Dropped);

    // But close_request (control) must be accepted even while closing.
    let close_policy = OverflowPolicy::Drop {
        mode: DropMode::DropOldestLowPriority,
    };
    let out2 = q
        .push(
            exchange_id,
            conn_id,
            QueuedOutbound::close_request(),
            &close_policy,
            ts,
        )
        .await
        .unwrap();
    assert_eq!(out2, ucel_transport::ws::priority::PushOutcome::Enqueued);
}
