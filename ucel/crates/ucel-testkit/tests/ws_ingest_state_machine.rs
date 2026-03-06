use ucel_core::{is_valid_transition, IngestFailureClass, IngestLifecycleState};
use ucel_testkit::ws_ingest::sample_key;
use ucel_transport::ws::supervisor::WsIngestSupervisor;

#[test]
fn state_machine_happy_and_failure_paths() {
    assert!(is_valid_transition(
        IngestLifecycleState::Planned,
        IngestLifecycleState::PendingConnect
    ));
    assert!(is_valid_transition(
        IngestLifecycleState::AwaitingAck,
        IngestLifecycleState::Active
    ));
    assert!(!is_valid_transition(
        IngestLifecycleState::Planned,
        IngestLifecycleState::Active
    ));

    let mut sup = WsIngestSupervisor::default();
    let d = sup.on_failure(
        sample_key("public"),
        IngestLifecycleState::Active,
        IngestFailureClass::HeartbeatTimeout,
    );
    assert!(matches!(d, ucel_core::IngestResumeDirective::Resubscribe));
}
