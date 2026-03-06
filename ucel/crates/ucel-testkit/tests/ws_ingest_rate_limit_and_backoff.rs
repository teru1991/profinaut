use ucel_core::{IngestFailureClass, IngestLifecycleState};
use ucel_testkit::ws_ingest::sample_key;
use ucel_transport::ws::backoff::ingest_backoff_ms;
use ucel_transport::ws::supervisor::WsIngestSupervisor;

#[test]
fn rate_limit_backoff_and_reason_tracking() {
    let b1 = ingest_backoff_ms(1, 100, 10_000, 0);
    let b2 = ingest_backoff_ms(2, 100, 10_000, 0);
    assert!(b2 >= b1);

    let mut sup = WsIngestSupervisor::default();
    let _ = sup.on_failure(
        sample_key("public"),
        IngestLifecycleState::Active,
        IngestFailureClass::RateLimited,
    );
    assert_eq!(sup.journal.events().len(), 1);
    assert_eq!(
        sup.journal.events()[0].failure,
        Some(IngestFailureClass::RateLimited)
    );
}
