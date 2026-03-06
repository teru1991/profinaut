use ucel_core::{
    CanonicalPrivateWsEvent, PrivateWsAckMode, PrivateWsLifecycleState, PrivateWsRejectClass,
};
use ucel_transport::ws::private_runtime::{PrivateWsEventEnvelope, PrivateWsSession, PrivateWsSessionConfig};

pub fn build_session(venue: &str, ack_mode: PrivateWsAckMode) -> PrivateWsSession {
    PrivateWsSession::new(PrivateWsSessionConfig {
        venue: venue.to_string(),
        requires_auth: true,
        ack_mode,
        auth_ack_timeout_ms: 5_000,
        sub_ack_timeout_ms: 5_000,
    })
}

pub fn reject_from_text(msg: &str) -> PrivateWsRejectClass {
    let m = msg.to_ascii_lowercase();
    if m.contains("auth") {
        PrivateWsRejectClass::AuthFailed
    } else if m.contains("entitlement") {
        PrivateWsRejectClass::EntitlementDenied
    } else if m.contains("expired") {
        PrivateWsRejectClass::SessionExpired
    } else {
        PrivateWsRejectClass::Unknown
    }
}

pub fn normalized_unknown_event() -> CanonicalPrivateWsEvent {
    CanonicalPrivateWsEvent::Unknown { channel: None }
}

pub fn envelope(payload: &str) -> PrivateWsEventEnvelope {
    PrivateWsEventEnvelope {
        channel: None,
        payload: payload.to_string(),
    }
}

pub fn ensure_active(session: &PrivateWsSession) -> bool {
    session.state == PrivateWsLifecycleState::Active
}
