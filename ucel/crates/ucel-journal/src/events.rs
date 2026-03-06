use serde::{Deserialize, Serialize};
use ucel_core::{IngestFailureClass, IngestLifecycleState, IngestResumeDirective, IngestStreamKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestJournalEvent {
    pub event_id: String,
    pub ts_ms: u64,
    pub key: IngestStreamKey,
    pub from: IngestLifecycleState,
    pub to: IngestLifecycleState,
    pub failure: Option<IngestFailureClass>,
    pub directive: Option<IngestResumeDirective>,
    pub detail: String,
}

pub fn sanitize_detail(input: &str) -> String {
    input
        .replace("token", "[redacted]")
        .replace("secret", "[redacted]")
        .replace("signature", "[redacted]")
}
