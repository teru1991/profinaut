use ucel_core::{IngestLifecycleState, IngestStreamKey};
use ucel_subscription_store::{resume_candidates, DurableIngestState, DurableStateStore};

#[test]
fn restart_rebuilds_resume_candidates() {
    let key = IngestStreamKey {
        exchange: "binance".into(),
        family: "spot".into(),
        channel: "orderbook".into(),
        symbol: "BTCUSDT".into(),
        shard: 0,
        auth_scope: "private".into(),
    };
    let mut store = DurableStateStore::default();
    store.upsert(DurableIngestState {
        key,
        lifecycle: IngestLifecycleState::Active,
        checkpoint: Default::default(),
        journal_event_id: Some("e1".into()),
    });
    assert_eq!(resume_candidates(&store).len(), 1);
}
