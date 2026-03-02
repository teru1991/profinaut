use ucel_testkit::chaos::run_throttle_scenario;

#[test]
fn retry_after_prevents_storm() {
    let c = run_throttle_scenario(1_000, 10);
    assert!(c.throttle_events <= 10);
    assert!(!c.panicked);
}
