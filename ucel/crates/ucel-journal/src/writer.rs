use crate::events::{sanitize_detail, IngestJournalEvent};

#[derive(Debug, Default)]
pub struct IngestJournalWriter {
    events: Vec<IngestJournalEvent>,
}

impl IngestJournalWriter {
    pub fn append(&mut self, mut event: IngestJournalEvent) {
        event.detail = sanitize_detail(&event.detail);
        self.events.push(event);
    }

    pub fn events(&self) -> &[IngestJournalEvent] {
        &self.events
    }
}
