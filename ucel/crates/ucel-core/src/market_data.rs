use crate::{Decimal, UcelError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalTicker {
    pub symbol: String,
    pub best_bid: Decimal,
    pub best_ask: Decimal,
    pub last_price: Decimal,
    pub ts_event: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalTrade {
    pub symbol: String,
    pub trade_id: String,
    pub price: Decimal,
    pub qty: Decimal,
    pub side: crate::Side,
    pub ts_event: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOrderBookLevel {
    pub price: Decimal,
    pub qty: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<CanonicalOrderBookLevel>,
    pub asks: Vec<CanonicalOrderBookLevel>,
    pub sequence: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOrderBookDelta {
    pub symbol: String,
    pub bids: Vec<CanonicalOrderBookLevel>,
    pub asks: Vec<CanonicalOrderBookLevel>,
    pub sequence_start: Option<u64>,
    pub sequence_end: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalCandle {
    pub symbol: String,
    pub interval: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub ts_open: u64,
    pub ts_close: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketDataChannel {
    Ticker,
    Trades,
    OrderBook,
    Candles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicAdapterSupport {
    Supported,
    Partial,
    NotSupported,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicWsAckMode {
    ExplicitAck,
    ImplicitObservation,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicWsIntegrityMode {
    None,
    SequenceOnly,
    Checksum,
    SequenceAndChecksum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicWsReasonCode {
    NotSupported,
    SubscriptionRejected,
    AckTimeout,
    HeartbeatTimeout,
    ChecksumMismatch,
    GapDetected,
    RateLimited,
    RetryableTransport,
}

pub fn validate_ticker(ticker: &CanonicalTicker) -> Result<(), UcelError> {
    if ticker.best_bid > ticker.best_ask {
        return Err(UcelError::new(
            crate::ErrorCode::Desync,
            "crossed book in ticker",
        ));
    }
    Ok(())
}

pub fn validate_candle(candle: &CanonicalCandle) -> Result<(), UcelError> {
    if candle.high < candle.low {
        return Err(UcelError::new(
            crate::ErrorCode::CatalogInvalid,
            "candle high must be >= low",
        ));
    }
    Ok(())
}

pub fn validate_trade(trade: &CanonicalTrade) -> Result<(), UcelError> {
    if trade.qty <= Decimal::from(0) {
        return Err(UcelError::new(
            crate::ErrorCode::CatalogInvalid,
            "trade qty must be positive",
        ));
    }
    Ok(())
}

pub fn apply_orderbook_delta(
    snapshot: &CanonicalOrderBookSnapshot,
    delta: &CanonicalOrderBookDelta,
) -> CanonicalOrderBookSnapshot {
    let mut next = snapshot.clone();
    for level in &delta.bids {
        upsert_level(&mut next.bids, level.clone(), true);
    }
    for level in &delta.asks {
        upsert_level(&mut next.asks, level.clone(), false);
    }
    next.sequence = delta.sequence_end.or(delta.sequence_start).or(snapshot.sequence);
    next
}

fn upsert_level(levels: &mut Vec<CanonicalOrderBookLevel>, level: CanonicalOrderBookLevel, bid: bool) {
    if level.qty <= Decimal::from(0) {
        levels.retain(|x| x.price != level.price);
        return;
    }
    if let Some(idx) = levels.iter().position(|x| x.price == level.price) {
        levels[idx] = level;
    } else {
        levels.push(level);
    }
    if bid {
        levels.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        levels.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));
    }
}

pub fn guard_orderbook(snapshot: &CanonicalOrderBookSnapshot) -> Result<(), UcelError> {
    let bid = snapshot.bids.first();
    let ask = snapshot.asks.first();
    if let (Some(bid), Some(ask)) = (bid, ask) {
        if bid.price > ask.price {
            return Err(UcelError::new(
                crate::ErrorCode::Desync,
                "orderbook crossed after normalization",
            ));
        }
    }
    Ok(())
}
