use serde::{Deserialize, Serialize};

use crate::AuthSurface;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsChannel {
    Balances,
    Orders,
    Fills,
    Positions,
    Session,
}

impl PrivateWsChannel {
    pub fn requires_auth(self) -> bool {
        true
    }

    pub fn auth_surface(self) -> AuthSurface {
        AuthSurface::PrivateWs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsSupport {
    Supported,
    Partial,
    NotSupported,
    BlockedByPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsAckMode {
    ExplicitAck,
    ImplicitObservation,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsLifecycleState {
    Connecting,
    Authenticating,
    Authenticated,
    Subscribing,
    Active,
    ReauthPending,
    ResubscribePending,
    Failed,
    Deadlettered,
}

impl PrivateWsLifecycleState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use PrivateWsLifecycleState::*;
        matches!(
            (self, next),
            (Connecting, Authenticating)
                | (Authenticating, Authenticated)
                | (Authenticating, Failed)
                | (Authenticated, Subscribing)
                | (Subscribing, Active)
                | (Subscribing, Failed)
                | (Active, ReauthPending)
                | (Active, ResubscribePending)
                | (ReauthPending, Authenticating)
                | (ResubscribePending, Subscribing)
                | (Failed, Deadlettered)
                | (_, Failed)
                | (_, Deadlettered)
        ) || self == next
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalBalanceEvent {
    pub asset: String,
    pub free: Option<String>,
    pub locked: Option<String>,
    pub ts_event_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalOrderEvent {
    pub order_id: String,
    pub symbol: String,
    pub side: Option<String>,
    pub status: String,
    pub price: Option<String>,
    pub qty: Option<String>,
    pub ts_event_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalFillEvent {
    pub fill_id: String,
    pub order_id: Option<String>,
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub price: Option<String>,
    pub qty: Option<String>,
    pub fee: Option<String>,
    pub ts_event_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalPositionEvent {
    pub symbol: String,
    pub side: Option<String>,
    pub qty: String,
    pub entry_price: Option<String>,
    pub liquidation_price: Option<String>,
    pub ts_event_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSessionEvent {
    pub status: String,
    pub message: Option<String>,
    pub ts_event_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalPrivateWsEvent {
    Balance(CanonicalBalanceEvent),
    Order(CanonicalOrderEvent),
    Fill(CanonicalFillEvent),
    Position(CanonicalPositionEvent),
    Session(CanonicalSessionEvent),
    Unknown { channel: Option<PrivateWsChannel> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsRejectClass {
    AuthFailed,
    EntitlementDenied,
    SessionExpired,
    AckTimeout,
    ReauthRequired,
    SubscriptionRejected,
    GapDetected,
    TransportClosed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateWsOutcome {
    Active,
    RetryableFailure,
    PermanentFailure,
    Deadletter,
}

impl PrivateWsRejectClass {
    pub fn retryable(self) -> bool {
        matches!(
            self,
            PrivateWsRejectClass::SessionExpired
                | PrivateWsRejectClass::AckTimeout
                | PrivateWsRejectClass::ReauthRequired
                | PrivateWsRejectClass::TransportClosed
                | PrivateWsRejectClass::GapDetected
        )
    }

    pub fn as_outcome(self) -> PrivateWsOutcome {
        match self {
            PrivateWsRejectClass::AuthFailed
            | PrivateWsRejectClass::EntitlementDenied
            | PrivateWsRejectClass::SubscriptionRejected => PrivateWsOutcome::PermanentFailure,
            PrivateWsRejectClass::Unknown => PrivateWsOutcome::Deadletter,
            _ if self.retryable() => PrivateWsOutcome::RetryableFailure,
            _ => PrivateWsOutcome::Deadletter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_guard_rejects_invalid_transition() {
        assert!(
            !PrivateWsLifecycleState::Connecting.can_transition_to(PrivateWsLifecycleState::Active)
        );
        assert!(
            PrivateWsLifecycleState::Subscribing.can_transition_to(PrivateWsLifecycleState::Active)
        );
    }

    #[test]
    fn reject_class_retryability() {
        assert!(PrivateWsRejectClass::AckTimeout.retryable());
        assert_eq!(
            PrivateWsRejectClass::AuthFailed.as_outcome(),
            PrivateWsOutcome::PermanentFailure
        );
    }
}
