use ucel_execution_core::idempotency::{make_client_order_id, MonotonicCounter};

#[test]
fn client_order_id_is_stable_and_safe_ascii() {
    let id = make_client_order_id("gmo", "run:1", 42);
    assert_eq!(id, "gmo_run_1_42");
}

#[test]
fn monotonic_counter_increments() {
    let c = MonotonicCounter::default();
    let a = c.next();
    let b = c.next();
    assert!(b > a);
}
