use std::time::{SystemTime, UNIX_EPOCH};

use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};

use ucel_transport::ws::heartbeat::StaleConfig;

/// unix seconds (i64)
fn now_unix_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tokio::test(flavor = "current_thread")]
async fn stale_detection_requeues_only_stale_active_subscriptions() {
    // Arrange
    let exchange_id = "bitbank";
    let conn_id = "conn-1";

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    let now = now_unix_i64();

    store
        .seed(
            &[
                SubscriptionRow {
                    key: "k_recent".into(),
                    exchange_id: exchange_id.into(),
                    op_id: "crypto.public.ticker".into(),
                    symbol: Some("BTC/JPY".into()),
                    params_json: "{}".into(),
                    assigned_conn: Some(conn_id.into()),
                },
                SubscriptionRow {
                    key: "k_stale".into(),
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

    // mark active
    store.mark_active("k_recent", now).unwrap();
    store.mark_active("k_stale", now).unwrap();

    // bump last message:
    // - recent gets a message now
    // - stale gets a message far in the past
    store.bump_last_message("k_recent", now).unwrap();
    store.bump_last_message("k_stale", now - 3600).unwrap(); // 1 hour ago

    // stale config: stale if > 30s without messages
    let cfg = StaleConfig {
        stale_after_secs: 30,
        max_batch: 100,
    };

    // Act:
    let changed = store
        .requeue_stale_active_to_pending(
            exchange_id,
            conn_id,
            cfg.stale_after_secs,
            cfg.max_batch,
            now,
        )
        .unwrap();

    // Assert:
    assert_eq!(changed, 1);

    let s_recent = store.state_of("k_recent").unwrap().unwrap();
    let s_stale = store.state_of("k_stale").unwrap().unwrap();

    assert_eq!(s_recent, "active");
    assert_eq!(s_stale, "pending");
}

#[tokio::test(flavor = "current_thread")]
async fn stale_detection_is_noop_when_all_recent() {
    let exchange_id = "x";
    let conn_id = "c1";
    let now = now_unix_i64();

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    store
        .seed(
            &[SubscriptionRow {
                key: "k1".into(),
                exchange_id: exchange_id.into(),
                op_id: "crypto.public.trades".into(),
                symbol: Some("BTC/USDT".into()),
                params_json: "{}".into(),
                assigned_conn: Some(conn_id.into()),
            }],
            now,
        )
        .unwrap();

    store.mark_active("k1", now).unwrap();
    store.bump_last_message("k1", now).unwrap();

    let cfg = StaleConfig {
        stale_after_secs: 60,
        max_batch: 100,
    };

    let changed = store
        .requeue_stale_active_to_pending(
            exchange_id,
            conn_id,
            cfg.stale_after_secs,
            cfg.max_batch,
            now,
        )
        .unwrap();

    assert_eq!(changed, 0);
    let s = store.state_of("k1").unwrap().unwrap();
    assert_eq!(s, "active");
}

#[tokio::test(flavor = "current_thread")]
async fn stale_detection_handles_null_last_message_as_stale_after_grace() {
    let exchange_id = "x";
    let conn_id = "c1";
    let now = now_unix_i64();

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    store
        .seed(
            &[SubscriptionRow {
                key: "k_null".into(),
                exchange_id: exchange_id.into(),
                op_id: "crypto.public.book".into(),
                symbol: Some("BTC/USDT".into()),
                params_json: "{}".into(),
                assigned_conn: Some(conn_id.into()),
            }],
            now,
        )
        .unwrap();

    // activated long ago, but no messages ever
    store.mark_active("k_null", now - 3600).unwrap();

    let cfg = StaleConfig {
        stale_after_secs: 30,
        max_batch: 100,
    };

    let changed = store
        .requeue_stale_active_to_pending(
            exchange_id,
            conn_id,
            cfg.stale_after_secs,
            cfg.max_batch,
            now,
        )
        .unwrap();

    assert_eq!(changed, 1);
    let s = store.state_of("k_null").unwrap().unwrap();
    assert_eq!(s, "pending");
}
