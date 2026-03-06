use serde::{Deserialize, Serialize};
use ucel_core::{IngestLifecycleState, IngestStreamKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredIngestStream {
    pub key: IngestStreamKey,
    pub state: IngestLifecycleState,
    pub requires_snapshot: bool,
    pub priority: u8,
}

pub fn build_desired_plan(keys: Vec<IngestStreamKey>) -> Vec<DesiredIngestStream> {
    keys.into_iter()
        .map(|key| DesiredIngestStream {
            requires_snapshot: key.channel.contains("orderbook") || key.channel.contains("book"),
            priority: if key.auth_scope == "private" { 0 } else { 1 },
            key,
            state: IngestLifecycleState::Planned,
        })
        .collect()
}
