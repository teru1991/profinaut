use crate::domain::{ArtifactRef, CanonicalEntityId, EntityAlias, Quality};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrProvider {
    Edinet,
    Sec,
    SecEdgar,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrEvent {
    pub provider: IrProvider,
    pub source_event_id: String,
    pub entity_id: CanonicalEntityId,
    pub entity_aliases: Vec<EntityAlias>,
    pub filing_type: String,
    pub filing_date: Option<String>,
    pub published_at: Option<u64>,
    pub observed_at: u64,
    pub artifacts: Vec<ArtifactRef>,
    pub quality: Quality,
    pub trace_id: String,
}

impl IrEvent {
    pub fn dedupe_key(&self) -> String {
        format!(
            "{:?}:{}:{}",
            self.provider,
            self.entity_id.as_key(),
            self.source_event_id
        )
    }
}
