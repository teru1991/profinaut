use ucel_core::CanonicalPrivateWsEvent;
use ucel_testkit::private_ws::normalized_unknown_event;

#[test]
fn unknown_event_does_not_panic() {
    let ev = normalized_unknown_event();
    assert!(matches!(ev, CanonicalPrivateWsEvent::Unknown { .. }));
}

#[test]
fn no_secret_literal_in_event_debug_preview() {
    let ev = normalized_unknown_event();
    let dbg = format!("{ev:?}");
    assert!(!dbg.to_ascii_lowercase().contains("token"));
    assert!(!dbg.to_ascii_lowercase().contains("secret"));
}
