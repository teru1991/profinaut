use ucel_core::{
    VendorPublicWsIntegrityMode, VendorPublicWsReadinessMode, VendorPublicWsResumeMode,
};
use ucel_transport::ws::ext_runtime::{next_resume_action, should_activate_now};

#[test]
fn readiness_mode_transitions_are_deterministic() {
    assert!(should_activate_now(
        VendorPublicWsReadinessMode::ExplicitAck,
        true,
        false
    ));
    assert!(!should_activate_now(
        VendorPublicWsReadinessMode::ExplicitAck,
        false,
        true
    ));
    assert!(should_activate_now(
        VendorPublicWsReadinessMode::ImplicitObservation,
        false,
        true
    ));
    assert!(should_activate_now(
        VendorPublicWsReadinessMode::ImmediateActive,
        false,
        false
    ));
}

#[test]
fn resume_and_integrity_modes_are_exposed() {
    assert_eq!(
        next_resume_action(VendorPublicWsResumeMode::ResubscribeOnly),
        "resubscribe_only"
    );
    assert_eq!(
        next_resume_action(VendorPublicWsResumeMode::ResnapshotThenResubscribe),
        "resnapshot_then_resubscribe"
    );
    assert_eq!(
        next_resume_action(VendorPublicWsResumeMode::Deadletter),
        "deadletter"
    );

    let modes = [
        VendorPublicWsIntegrityMode::None,
        VendorPublicWsIntegrityMode::SnapshotOnly,
        VendorPublicWsIntegrityMode::SequenceOnly,
        VendorPublicWsIntegrityMode::ChecksumOnly,
        VendorPublicWsIntegrityMode::SequenceAndChecksum,
    ];
    assert_eq!(modes.len(), 5);
}
