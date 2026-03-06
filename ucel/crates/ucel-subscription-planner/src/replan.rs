use crate::plan::DesiredIngestStream;
use ucel_core::{IngestLifecycleState, IngestResumeDirective};

pub fn replan_for_resume(
    streams: &[DesiredIngestStream],
    directive: IngestResumeDirective,
) -> Vec<DesiredIngestStream> {
    streams
        .iter()
        .filter(|s| s.state != IngestLifecycleState::Deadlettered)
        .map(|s| {
            let mut next = s.clone();
            next.state = IngestLifecycleState::ResumePending;
            if matches!(directive, IngestResumeDirective::ResnapshotThenResubscribe) {
                next.requires_snapshot = true;
            }
            next
        })
        .collect()
}
