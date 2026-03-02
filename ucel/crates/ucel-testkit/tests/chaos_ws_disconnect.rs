use ucel_testkit::chaos::run_disconnect_scenario;

#[test]
fn reconnects_and_returns_online() {
    let c = run_disconnect_scenario(3, 8);
    assert!(c.reconnects <= 8);
    assert!(c.online_events >= 1);
    assert!(!c.panicked);
}
