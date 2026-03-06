use ucel_core::{PrivateWsAckMode, PrivateWsLifecycleState, PrivateWsRejectClass};
use ucel_testkit::private_ws::{build_session, reject_from_text};

#[test]
fn explicit_ack_moves_to_active_after_acks() {
    let mut s = build_session("bitbank", PrivateWsAckMode::ExplicitAck);
    s.on_connected();
    s.on_auth_ack();
    s.on_subscribe_sent();
    assert_eq!(s.state, PrivateWsLifecycleState::Subscribing);
    s.on_sub_ack();
    assert_eq!(s.state, PrivateWsLifecycleState::Active);
}

#[test]
fn implicit_observation_moves_to_active_on_first_event() {
    let mut s = build_session("coincheck", PrivateWsAckMode::ImplicitObservation);
    s.on_connected();
    s.on_auth_ack();
    s.on_subscribe_sent();
    s.on_first_valid_event();
    assert_eq!(s.state, PrivateWsLifecycleState::Active);
}

#[test]
fn auth_reject_is_normalized() {
    assert_eq!(
        reject_from_text("auth failed"),
        PrivateWsRejectClass::AuthFailed
    );
}
