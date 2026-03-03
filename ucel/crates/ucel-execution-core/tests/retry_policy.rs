use ucel_execution_core::retry_policy::{decide, OpKind, RetryClass};

#[test]
fn place_order_without_idempotency_is_not_retryable_on_timeout() {
    let d = decide(OpKind::PlaceOrder, None, true, false);
    assert_eq!(d.class, RetryClass::NonRetryable);
}

#[test]
fn place_order_with_idempotency_is_retryable_on_5xx() {
    let d = decide(OpKind::PlaceOrder, Some(502), false, true);
    assert_eq!(d.class, RetryClass::Retryable);
}

#[test]
fn rate_limit_is_cooldown() {
    let d = decide(OpKind::ReadOnly, Some(429), false, false);
    assert_eq!(d.class, RetryClass::Cooldown);
}
