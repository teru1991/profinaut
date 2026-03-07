use ucel_core::{PublicWsAckMode, PublicWsIntegrityMode};
use ucel_transport::ws::public_runtime::{
    signal_to_deadletter_reason, DomesticPublicWsDeadletterReason, DomesticPublicWsReadyState,
    PublicRuntimeSignal, PublicWsSession, PublicWsSessionConfig, PublicWsSubscribePlan,
};

#[test]
fn ack_mode_transitions_are_handled() {
    let mut explicit = PublicWsSession::new(PublicWsSessionConfig {
        ack_mode: PublicWsAckMode::ExplicitAck,
        integrity_mode: PublicWsIntegrityMode::None,
        heartbeat_timeout_ms: 5_000,
        ack_timeout_ms: 1_000,
    });
    explicit.mark_subscribing(PublicWsSubscribePlan {
        channel: "public_ticker".into(),
        symbol: "btc_jpy".into(),
    });
    assert_eq!(explicit.ready_state, DomesticPublicWsReadyState::Subscribed);
    explicit.mark_active("public_ticker", "btc_jpy");
    assert_eq!(explicit.ready_state, DomesticPublicWsReadyState::Active);

    let mut implicit = PublicWsSession::new(PublicWsSessionConfig {
        ack_mode: PublicWsAckMode::ImplicitObservation,
        integrity_mode: PublicWsIntegrityMode::None,
        heartbeat_timeout_ms: 5_000,
        ack_timeout_ms: 1_000,
    });
    implicit.mark_subscribing(PublicWsSubscribePlan {
        channel: "public_trades".into(),
        symbol: "btc_jpy".into(),
    });
    implicit.observe_event_ready_if_needed();
    assert_eq!(implicit.ready_state, DomesticPublicWsReadyState::Active);

    let mut immediate = PublicWsSession::new(PublicWsSessionConfig {
        ack_mode: PublicWsAckMode::None,
        integrity_mode: PublicWsIntegrityMode::None,
        heartbeat_timeout_ms: 5_000,
        ack_timeout_ms: 1_000,
    });
    immediate.mark_subscribing(PublicWsSubscribePlan {
        channel: "public_orderbook".into(),
        symbol: "btc_jpy".into(),
    });
    immediate.activate_immediately_if_needed();
    assert_eq!(immediate.ready_state, DomesticPublicWsReadyState::Active);
}

#[test]
fn deadletter_reason_mapping_is_stable() {
    assert_eq!(
        signal_to_deadletter_reason(PublicRuntimeSignal::HeartbeatTimeout),
        Some(DomesticPublicWsDeadletterReason::HeartbeatTimeout)
    );
    assert_eq!(
        signal_to_deadletter_reason(PublicRuntimeSignal::GapDetected),
        Some(DomesticPublicWsDeadletterReason::GapDetected)
    );
    assert_eq!(
        signal_to_deadletter_reason(PublicRuntimeSignal::AckObserved),
        None
    );
}
