use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};

fn now() -> i64 {
    1000
}

#[test]
fn pending_batch_skips_cooldown_items() {
    let mut store = SubscriptionStore::open(":memory:").unwrap();
    store
        .seed(
            &[SubscriptionRow {
                key: "k1".into(),
                exchange_id: "x".into(),
                op_id: "op".into(),
                symbol: None,
                params_json: "{}".into(),
                assigned_conn: Some("c".into()),
            }],
            now(),
        )
        .unwrap();

    // cooldown until 2000
    store.apply_rate_limit_cooldown("k1", now(), 1000).unwrap();

    let got = store.next_pending_batch("x", "c", 10, now()).unwrap();
    assert_eq!(got.len(), 0);

    // after cooldown passes
    let got2 = store.next_pending_batch("x", "c", 10, 2500).unwrap();
    assert_eq!(got2.len(), 1);
}
