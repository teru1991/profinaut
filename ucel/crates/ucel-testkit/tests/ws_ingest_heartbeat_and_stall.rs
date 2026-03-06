use std::time::{Duration, Instant};
use ucel_transport::ws::heartbeat::{HeartbeatConfig, HeartbeatTracker};

#[test]
fn heartbeat_and_stall_detection() {
    let start = Instant::now();
    let mut tracker = HeartbeatTracker::new(
        HeartbeatConfig {
            ping_interval_secs: 1,
            idle_timeout_secs: 2,
        },
        start,
    );
    tracker.observe_recv(start + Duration::from_millis(500));
    assert!(!tracker.is_stale(start + Duration::from_secs(1)));
    assert!(tracker.is_stale(start + Duration::from_secs(3)));
}
