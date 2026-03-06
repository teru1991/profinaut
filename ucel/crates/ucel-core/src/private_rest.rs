use serde::{Deserialize, Serialize};

use crate::{AuthSurface, UcelError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateRestOperation {
    GetBalances,
    GetOpenOrders,
    GetOrder,
    CancelOrder,
    GetFills,
    GetAccountProfile,
    GetPositions,
}

impl PrivateRestOperation {
    pub fn is_write(self) -> bool {
        matches!(self, PrivateRestOperation::CancelOrder)
    }

    pub fn auth_surface(self) -> AuthSurface {
        AuthSurface::PrivateRest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateRestSupport {
    Supported,
    Partial,
    NotSupported,
    BlockedByPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrySafety {
    SafeToRetry,
    UnsafeToRetry,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VenueRejectClass {
    Unauthorized,
    Forbidden,
    PermissionDenied,
    ValidationFailed,
    InsufficientFunds,
    NotFound,
    RateLimited,
    RetryableTransport,
    PermanentVenueReject,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateReadRequest {
    pub venue: String,
    pub operation: PrivateRestOperation,
    pub key_id: String,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateWriteRequest {
    pub venue: String,
    pub operation: PrivateRestOperation,
    pub key_id: String,
    pub order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub price: Option<String>,
    pub qty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalFill {
    pub fill_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: String,
    pub qty: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPosition {
    pub symbol: String,
    pub side: String,
    pub qty: String,
    pub entry_price: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalAccountProfile {
    pub account_id: Option<String>,
    pub status: Option<String>,
    pub raw_tier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOutcome {
    pub accepted: bool,
    pub venue_order_id: Option<String>,
    pub retry_safety: RetrySafety,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrivateRestResult<T> {
    Ok(T),
    VenueReject {
        class: VenueRejectClass,
        retry_safety: RetrySafety,
        message: String,
    },
    TransportError(UcelError),
}

pub fn normalize_reject_class(status: u16, message: &str, is_write: bool) -> VenueRejectClass {
    let m = message.to_ascii_lowercase();
    if status == 401 {
        return VenueRejectClass::Unauthorized;
    }
    if status == 403 {
        return VenueRejectClass::Forbidden;
    }
    if status == 404 {
        return VenueRejectClass::NotFound;
    }
    if status == 429 {
        return VenueRejectClass::RateLimited;
    }
    if status == 422 || m.contains("invalid") || m.contains("validation") {
        return VenueRejectClass::ValidationFailed;
    }
    if m.contains("insufficient") || m.contains("balance") || m.contains("fund") {
        return VenueRejectClass::InsufficientFunds;
    }
    if status >= 500 && !is_write {
        return VenueRejectClass::RetryableTransport;
    }
    if m.contains("not supported") {
        return VenueRejectClass::NotSupported;
    }
    VenueRejectClass::PermanentVenueReject
}

pub fn retry_safety_for(operation: PrivateRestOperation, status: u16) -> RetrySafety {
    if operation.is_write() {
        return RetrySafety::UnsafeToRetry;
    }
    if status == 429 || status >= 500 {
        RetrySafety::SafeToRetry
    } else {
        RetrySafety::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_is_write_and_not_retry_safe() {
        assert!(PrivateRestOperation::CancelOrder.is_write());
        assert_eq!(
            retry_safety_for(PrivateRestOperation::CancelOrder, 500),
            RetrySafety::UnsafeToRetry
        );
    }

    #[test]
    fn read_5xx_maps_to_retryable_transport() {
        assert_eq!(
            normalize_reject_class(503, "upstream unavailable", false),
            VenueRejectClass::RetryableTransport
        );
        assert_eq!(
            retry_safety_for(PrivateRestOperation::GetBalances, 503),
            RetrySafety::SafeToRetry
        );
    }
}
