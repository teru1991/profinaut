use std::sync::Arc;
use std::time::Duration;

use ucel_transport::ws::overflow::{DropMode, OverflowPolicy, Spooler, SpoolerConfig};
use ucel_transport::ws::priority::{
    OutboundPriority, PriorityQueue, PushOutcome, QueuedOutbound, WsOutboundFrame,
};

fn mk_item(p: OutboundPriority, op: &str, payload: &str) -> QueuedOutbound {
    QueuedOutbound {
        priority: p,
        op_id: Some(op.to_string()),
        symbol: None,
        frame: WsOutboundFrame::Text(payload.to_string()),
        meta: serde_json::json!({"op": op}),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn overflow_drop_is_deterministic_drop_newest() {
    let q = PriorityQueue::new(1);
    let exchange_id = "x";
    let conn_id = "c";
    let ts = 1u64;

    // Fill
    let out0 = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Public, "op.public.1", "A"),
            &OverflowPolicy::drop_newest(),
            ts,
        )
        .await
        .unwrap();
    assert_eq!(out0, PushOutcome::Enqueued);

    // Overflow: DropNewest => Dropped
    let out1 = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Public, "op.public.2", "B"),
            &OverflowPolicy::drop_newest(),
            ts,
        )
        .await
        .unwrap();
    assert_eq!(out1, PushOutcome::Dropped);

    // Ensure original remains
    let got = q.recv().await.unwrap();
    assert_eq!(got.frame.to_bytes(), b"A");
}

#[tokio::test(flavor = "current_thread")]
async fn overflow_drop_is_deterministic_drop_oldest_low_priority() {
    let q = PriorityQueue::new(1);
    let exchange_id = "x";
    let conn_id = "c";
    let ts = 1u64;

    // Fill with public
    q.push(
        exchange_id,
        conn_id,
        mk_item(OutboundPriority::Public, "op.public.1", "A"),
        &OverflowPolicy::drop_newest(),
        ts,
    )
        .await
        .unwrap();

    // Overflow with private: DropOldestLowPriority must evict public and accept private.
    let policy = OverflowPolicy::Drop {
        mode: DropMode::DropOldestLowPriority,
    };
    let out = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Private, "op.private.1", "P"),
            &policy,
            ts,
        )
        .await
        .unwrap();
    assert_eq!(out, PushOutcome::Enqueued);

    // Private comes out
    let got = q.recv().await.unwrap();
    assert_eq!(got.frame.to_bytes(), b"P");
}

#[tokio::test(flavor = "current_thread")]
async fn overflow_slowdown_waits_then_enqueues_when_space_frees() {
    let q = PriorityQueue::new(1);
    let exchange_id = "x";
    let conn_id = "c";
    let ts = 1u64;

    // Fill queue
    q.push(
        exchange_id,
        conn_id,
        mk_item(OutboundPriority::Public, "op.public.1", "A"),
        &OverflowPolicy::drop_newest(),
        ts,
    )
        .await
        .unwrap();

    // After 50ms, receiver pops one item => space becomes available.
    let q2 = q.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = q2.recv().await;
    });

    // SlowDown should wait up to 200ms, and then enqueue.
    let policy = OverflowPolicy::SlowDown {
        max_wait: Duration::from_millis(200),
        fallback: DropMode::DropNewest,
    };

    let out = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Private, "op.private.1", "P"),
            &policy,
            ts,
        )
        .await
        .unwrap();

    assert_eq!(out, PushOutcome::Enqueued);

    // The next item should be P (because A was already popped)
    let got = q.recv().await.unwrap();
    assert_eq!(got.frame.to_bytes(), b"P");
}

#[tokio::test(flavor = "current_thread")]
async fn overflow_slowdown_falls_back_when_no_space() {
    let q = PriorityQueue::new(1);
    let exchange_id = "x";
    let conn_id = "c";
    let ts = 1u64;

    // Fill
    q.push(
        exchange_id,
        conn_id,
        mk_item(OutboundPriority::Public, "op.public.1", "A"),
        &OverflowPolicy::drop_newest(),
        ts,
    )
        .await
        .unwrap();

    // No receiver => no space ever frees during wait.
    let policy = OverflowPolicy::SlowDown {
        max_wait: Duration::from_millis(50),
        fallback: DropMode::DropNewest,
    };

    let out = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Public, "op.public.2", "B"),
            &policy,
            ts,
        )
        .await
        .unwrap();

    assert_eq!(out, PushOutcome::Dropped);

    // Ensure original remains
    let got = q.recv().await.unwrap();
    assert_eq!(got.frame.to_bytes(), b"A");
}

#[tokio::test(flavor = "current_thread")]
async fn overflow_spill_to_disk_spills_and_is_observable() {
    let dir = tempfile::tempdir().unwrap();
    let sp = Arc::new(Spooler::open(SpoolerConfig::new(dir.path())).unwrap());

    let q = PriorityQueue::new(1);
    let exchange_id = "x";
    let conn_id = "c";
    let ts = 123u64;

    // Fill
    q.push(
        exchange_id,
        conn_id,
        mk_item(OutboundPriority::Public, "op.public.1", "A"),
        &OverflowPolicy::drop_newest(),
        ts,
    )
        .await
        .unwrap();

    // Spill
    let policy = OverflowPolicy::SpillToDisk {
        spooler: sp,
        fallback: DropMode::DropNewest,
    };

    let out = q
        .push(
            exchange_id,
            conn_id,
            mk_item(OutboundPriority::Public, "op.public.2", "B"),
            &policy,
            ts,
        )
        .await
        .unwrap();

    assert_eq!(out, PushOutcome::Spilled);

    // A remains in queue, B is spilled
    let got = q.recv().await.unwrap();
    assert_eq!(got.frame.to_bytes(), b"A");

    // Verify a file exists in spool dir
    let files: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert!(!files.is_empty());
}