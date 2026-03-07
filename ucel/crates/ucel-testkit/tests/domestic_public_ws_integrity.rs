use ucel_core::PublicWsReasonCode;
use ucel_transport::ws::integrity::{
    check_checksum, check_crossed_book, check_negative_qty, check_sequence,
};

#[test]
fn detects_integrity_failures() {
    assert_eq!(
        check_sequence(Some(10), 10).expect_err("duplicate sequence must fail"),
        PublicWsReasonCode::GapDetected
    );
    assert_eq!(
        check_checksum(Some("a"), Some("b")).expect_err("checksum mismatch must fail"),
        PublicWsReasonCode::ChecksumMismatch
    );
    assert_eq!(
        check_crossed_book(Some(101.0), Some(100.0)).expect_err("crossed book must fail"),
        PublicWsReasonCode::ChecksumMismatch
    );
    assert_eq!(
        check_negative_qty(true).expect_err("negative qty must fail"),
        PublicWsReasonCode::ChecksumMismatch
    );
}

#[test]
fn none_mode_equivalent_checks_can_pass() {
    check_sequence(None, 1).expect("no previous sequence should pass");
    check_checksum(None, None).expect("missing checksum not checked in none mode");
    check_crossed_book(Some(100.0), Some(100.0)).expect("flat top book is okay");
    check_negative_qty(false).expect("non-negative qty should pass");
}
