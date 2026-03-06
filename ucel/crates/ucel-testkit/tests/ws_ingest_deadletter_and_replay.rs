use ucel_core::{IngestFailureClass, IngestLifecycleState};
use ucel_testkit::ws_ingest::sample_key;
use ucel_transport::ws::supervisor::WsIngestSupervisor;

#[test]
fn deadletter_and_journal_redaction_replayable() {
    let mut sup = WsIngestSupervisor::default();
    let directive = sup.on_failure(
        sample_key("private"),
        IngestLifecycleState::AwaitingAuth,
        IngestFailureClass::PolicyBlocked,
    );
    assert!(matches!(
        directive,
        ucel_core::IngestResumeDirective::Deadletter
    ));
    let details = &sup.journal.events()[0].detail;
    assert!(!details.contains("token"));
}
