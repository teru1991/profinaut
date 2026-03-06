use ucel_core::{IngestFailureClass, IngestResumeDirective};
use ucel_testkit::ws_ingest::{can_escalate_checksum, mismatch_directive};

#[test]
fn checksum_gap_behaviour() {
    assert!(!can_escalate_checksum(1));
    assert!(can_escalate_checksum(2));
    assert!(matches!(
        mismatch_directive(),
        IngestResumeDirective::ResnapshotThenResubscribe
    ));
    let d = ucel_core::failure_to_resume_directive(
        IngestFailureClass::GapDetected,
        ucel_core::IngestIntegrityMode::None,
    );
    assert!(matches!(d, IngestResumeDirective::Resubscribe));
}
