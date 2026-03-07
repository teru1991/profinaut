use crate::{PublicWsAckMode, PublicWsIntegrityMode, PublicWsReasonCode, UcelError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicWsSurface {
    SubscribeTicker,
    SubscribeTrades,
    SubscribeOrderbook,
    SubscribeCandles,
    SubscribeSystemStatus,
    SubscribeMaintenanceStatus,
    SubscribeAssetStatus,
    SubscribeNetworkStatus,
    SubscribePublicDerivativeReference,
    SubscribePublicFundingReference,
    SubscribePublicOpenInterestReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicWsSupport {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsTickerEvent {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsTradeEvent {
    pub symbol: String,
    pub trade_id: String,
    pub price: String,
    pub qty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsOrderbookSnapshotEvent {
    pub symbol: String,
    pub sequence: Option<u64>,
    pub bid_levels: usize,
    pub ask_levels: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsOrderbookDeltaEvent {
    pub symbol: String,
    pub sequence_start: Option<u64>,
    pub sequence_end: Option<u64>,
    pub bid_levels: usize,
    pub ask_levels: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsCandleEvent {
    pub symbol: String,
    pub interval: Option<String>,
    pub close: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsSystemStatusEvent {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsMaintenanceStatusEvent {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsAssetStatusEvent {
    pub asset: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsNetworkStatusEvent {
    pub network: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsDerivativeReferenceEvent {
    pub symbol: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsFundingReferenceEvent {
    pub symbol: Option<String>,
    pub funding_rate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalWsOpenInterestReferenceEvent {
    pub symbol: Option<String>,
    pub open_interest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum CanonicalPublicWsEvent {
    Ticker(CanonicalWsTickerEvent),
    Trade(CanonicalWsTradeEvent),
    OrderbookSnapshot(CanonicalWsOrderbookSnapshotEvent),
    OrderbookDelta(CanonicalWsOrderbookDeltaEvent),
    Candle(CanonicalWsCandleEvent),
    SystemStatus(CanonicalWsSystemStatusEvent),
    MaintenanceStatus(CanonicalWsMaintenanceStatusEvent),
    AssetStatus(CanonicalWsAssetStatusEvent),
    NetworkStatus(CanonicalWsNetworkStatusEvent),
    DerivativeReference(CanonicalWsDerivativeReferenceEvent),
    FundingReference(CanonicalWsFundingReferenceEvent),
    OpenInterestReference(CanonicalWsOpenInterestReferenceEvent),
}

pub fn ws_surface_runtime_requirements(
    surface: PublicWsSurface,
) -> (PublicWsAckMode, PublicWsIntegrityMode) {
    match surface {
        PublicWsSurface::SubscribeTicker
        | PublicWsSurface::SubscribeTrades
        | PublicWsSurface::SubscribeCandles => (
            PublicWsAckMode::ImplicitObservation,
            PublicWsIntegrityMode::None,
        ),
        PublicWsSurface::SubscribeOrderbook => (
            PublicWsAckMode::ImplicitObservation,
            PublicWsIntegrityMode::SequenceAndChecksum,
        ),
        PublicWsSurface::SubscribeSystemStatus
        | PublicWsSurface::SubscribeMaintenanceStatus
        | PublicWsSurface::SubscribeAssetStatus
        | PublicWsSurface::SubscribeNetworkStatus
        | PublicWsSurface::SubscribePublicDerivativeReference
        | PublicWsSurface::SubscribePublicFundingReference
        | PublicWsSurface::SubscribePublicOpenInterestReference => {
            (PublicWsAckMode::None, PublicWsIntegrityMode::None)
        }
    }
}

pub fn validate_orderbook_integrity(
    bid_top: Option<f64>,
    ask_top: Option<f64>,
    negative_qty_found: bool,
    duplicate_sequence: bool,
) -> Result<(), PublicWsReasonCode> {
    if negative_qty_found {
        return Err(PublicWsReasonCode::ChecksumMismatch);
    }
    if duplicate_sequence {
        return Err(PublicWsReasonCode::GapDetected);
    }
    if let (Some(bid), Some(ask)) = (bid_top, ask_top) {
        if bid > ask {
            return Err(PublicWsReasonCode::ChecksumMismatch);
        }
    }
    Ok(())
}

pub fn validate_status_event_nonempty(status: Option<&str>) -> Result<(), UcelError> {
    match status {
        Some(s) if !s.trim().is_empty() => Ok(()),
        _ => Err(UcelError::new(
            crate::ErrorCode::CatalogInvalid,
            "status event must include non-empty status",
        )),
    }
}
