use crate::state::{DurableIngestState, DurableStateStore};
use ucel_core::IngestLifecycleState;

pub fn resume_candidates(store: &DurableStateStore) -> Vec<DurableIngestState> {
    store
        .streams
        .values()
        .filter(|s| {
            matches!(
                s.lifecycle,
                IngestLifecycleState::Active
                    | IngestLifecycleState::AwaitingAck
                    | IngestLifecycleState::PendingConnect
                    | IngestLifecycleState::ReconnectScheduled
                    | IngestLifecycleState::ResumePending
            )
        })
        .cloned()
        .collect()
}
