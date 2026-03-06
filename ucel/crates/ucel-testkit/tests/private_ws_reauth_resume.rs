use ucel_core::{PrivateWsAckMode, PrivateWsLifecycleState};
use ucel_testkit::private_ws::build_session;

#[test]
fn reauth_and_resume_transitions_are_stable() {
    let mut s = build_session("gmocoin", PrivateWsAckMode::ExplicitAck);
    s.on_connected();
    s.on_auth_ack();
    s.on_subscribe_sent();
    s.on_sub_ack();
    s.on_reauth_required();
    assert_eq!(s.state, PrivateWsLifecycleState::ReauthPending);
    s.on_reconnect();
    assert_eq!(s.state, PrivateWsLifecycleState::ResubscribePending);
}

#[test]
fn failed_state_can_be_reached() {
    let mut s = build_session("bitflyer", PrivateWsAckMode::ExplicitAck);
    s.fail();
    assert_eq!(s.state, PrivateWsLifecycleState::Failed);
}
