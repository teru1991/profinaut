use ucel_core::{PrivateRestOperation, RetrySafety, VenueRejectClass};
use ucel_testkit::private_rest::normalize_reason;

#[test]
fn private_rest_reason_codes_are_normalized() {
    let (class_401, _) = normalize_reason(401, "unauthorized", PrivateRestOperation::GetBalances);
    assert_eq!(class_401, VenueRejectClass::Unauthorized);

    let (class_funds, _) = normalize_reason(
        400,
        "insufficient balance",
        PrivateRestOperation::CancelOrder,
    );
    assert_eq!(class_funds, VenueRejectClass::InsufficientFunds);

    let (class_429, _) = normalize_reason(429, "rate limited", PrivateRestOperation::GetFills);
    assert_eq!(class_429, VenueRejectClass::RateLimited);

    let (class_validation, _) =
        normalize_reason(422, "validation error", PrivateRestOperation::GetOrder);
    assert_eq!(class_validation, VenueRejectClass::ValidationFailed);

    let (_, write_retry) = normalize_reason(
        503,
        "upstream unavailable",
        PrivateRestOperation::CancelOrder,
    );
    assert_eq!(write_retry, RetrySafety::UnsafeToRetry);

    let (read_class, read_retry) = normalize_reason(
        503,
        "upstream unavailable",
        PrivateRestOperation::GetOpenOrders,
    );
    assert_eq!(read_class, VenueRejectClass::RetryableTransport);
    assert_eq!(read_retry, RetrySafety::SafeToRetry);
}
