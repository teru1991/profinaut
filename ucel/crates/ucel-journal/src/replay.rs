use crate::events::IngestJournalEvent;
use std::collections::BTreeMap;
use ucel_core::{IngestLifecycleState, IngestStreamKey};

pub fn replay_last_state(
    events: &[IngestJournalEvent],
) -> BTreeMap<IngestStreamKey, IngestLifecycleState> {
    let mut map = BTreeMap::new();
    for event in events {
        map.insert(event.key.clone(), event.to);
    }
    map
}
