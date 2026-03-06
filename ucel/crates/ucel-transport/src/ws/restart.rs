use ucel_subscription_store::{resume_candidates, DurableIngestState, DurableStateStore};

pub fn rebuild_resume_plan(store: &DurableStateStore) -> Vec<DurableIngestState> {
    resume_candidates(store)
}
