use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ucel_core::{IngestCheckpoint, IngestLifecycleState, IngestStreamKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableIngestState {
    pub key: IngestStreamKey,
    pub lifecycle: IngestLifecycleState,
    pub checkpoint: IngestCheckpoint,
    pub journal_event_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DurableStateStore {
    pub streams: BTreeMap<String, DurableIngestState>,
}

pub fn stream_id(key: &IngestStreamKey) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        key.exchange, key.family, key.channel, key.symbol, key.shard, key.auth_scope
    )
}

impl DurableStateStore {
    pub fn upsert(&mut self, state: DurableIngestState) {
        self.streams.insert(stream_id(&state.key), state);
    }

    pub fn get(&self, key: &IngestStreamKey) -> Option<&DurableIngestState> {
        self.streams.get(&stream_id(key))
    }
}
