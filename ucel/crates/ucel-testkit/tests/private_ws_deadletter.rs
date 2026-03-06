use ucel_core::{PrivateWsRejectClass, PrivateWsOutcome};

#[test]
fn ack_timeout_is_retryable_failure() {
    assert_eq!(
        PrivateWsRejectClass::AckTimeout.as_outcome(),
        PrivateWsOutcome::RetryableFailure
    );
}

#[test]
fn subscription_reject_is_permanent_failure() {
    assert_eq!(
        PrivateWsRejectClass::SubscriptionRejected.as_outcome(),
        PrivateWsOutcome::PermanentFailure
    );
}

#[test]
fn gap_detected_is_retryable_failure() {
    assert_eq!(
        PrivateWsRejectClass::GapDetected.as_outcome(),
        PrivateWsOutcome::RetryableFailure
    );
}
