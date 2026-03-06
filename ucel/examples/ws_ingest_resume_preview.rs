use ucel_core::{IngestLifecycleState, IngestStreamKey};
use ucel_subscription_store::{resume_candidates, DurableIngestState, DurableStateStore};

fn main() {
    let mut store = DurableStateStore::default();
    store.upsert(DurableIngestState {
        key: IngestStreamKey {
            exchange: "binance".into(),
            family: "spot".into(),
            channel: "orderbook".into(),
            symbol: "BTCUSDT".into(),
            shard: 0,
            auth_scope: "private".into(),
        },
        lifecycle: IngestLifecycleState::Active,
        checkpoint: Default::default(),
        journal_event_id: None,
    });
    println!("resume_candidates={}", resume_candidates(&store).len());
}
