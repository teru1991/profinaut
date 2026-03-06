use std::time::{SystemTime, UNIX_EPOCH};
use ucel_core::{
    failure_to_resume_directive, IngestFailureClass, IngestLifecycleState, IngestResumeDirective,
    IngestStreamKey,
};
use ucel_journal::{IngestJournalEvent, IngestJournalWriter};
use ucel_subscription_store::{DurableIngestState, DurableStateStore};

#[derive(Debug, Default)]
pub struct WsIngestSupervisor {
    pub store: DurableStateStore,
    pub journal: IngestJournalWriter,
}

impl WsIngestSupervisor {
    pub fn transition(
        &mut self,
        key: IngestStreamKey,
        from: IngestLifecycleState,
        to: IngestLifecycleState,
        failure: Option<IngestFailureClass>,
        directive: Option<IngestResumeDirective>,
        detail: &str,
    ) {
        let ts_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let event = IngestJournalEvent {
            event_id: format!("{}-{}", key.exchange, ts_ms),
            ts_ms,
            key: key.clone(),
            from,
            to,
            failure,
            directive,
            detail: detail.to_string(),
        };
        self.journal.append(event);
        self.store.upsert(DurableIngestState {
            key,
            lifecycle: to,
            checkpoint: Default::default(),
            journal_event_id: self.journal.events().last().map(|e| e.event_id.clone()),
        });
    }

    pub fn on_failure(
        &mut self,
        key: IngestStreamKey,
        from: IngestLifecycleState,
        failure: IngestFailureClass,
    ) -> IngestResumeDirective {
        let directive =
            failure_to_resume_directive(failure, ucel_core::IngestIntegrityMode::Sequence);
        let to = if matches!(directive, IngestResumeDirective::Deadletter) {
            IngestLifecycleState::Deadlettered
        } else {
            IngestLifecycleState::ReconnectScheduled
        };
        self.transition(
            key,
            from,
            to,
            Some(failure),
            Some(directive),
            "failure handled by supervisor",
        );
        directive
    }
}
