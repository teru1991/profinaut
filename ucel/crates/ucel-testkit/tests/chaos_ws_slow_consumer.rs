use ucel_testkit::chaos::run_slow_consumer_scenario;

#[test]
fn backpressure_drops_without_panic() {
    let c = run_slow_consumer_scenario(100, 1_000);
    assert!(c.dropped_frames > 0);
    assert_eq!(c.online_events, 1);
    assert!(!c.panicked);
}
