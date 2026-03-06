use ucel_core::{PublicWsAckMode, PublicWsIntegrityMode};
use ucel_transport::ws::public_runtime::{
    PublicWsSession, PublicWsSessionConfig, PublicWsSubscribePlan,
};

#[test]
fn reconnect_resume_uses_active_subscriptions() {
    let mut session = PublicWsSession::new(PublicWsSessionConfig {
        ack_mode: PublicWsAckMode::ExplicitAck,
        integrity_mode: PublicWsIntegrityMode::SequenceOnly,
        heartbeat_timeout_ms: 5_000,
        ack_timeout_ms: 1_000,
    });
    session.mark_subscribing(PublicWsSubscribePlan {
        channel: "trades".into(),
        symbol: "BTCUSDT".into(),
    });
    session.mark_active("trades", "BTCUSDT");
    let plan = session.resume_plan();
    assert_eq!(plan.subscriptions.len(), 1);
}
