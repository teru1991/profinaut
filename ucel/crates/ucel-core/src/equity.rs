use crate::{ErrorCode, UcelError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquityMarket {
    JP,
    US,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EquityExchangeCode(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquityLatencyClass {
    Realtime,
    Delayed,
    EndOfDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquityAdjustmentMode {
    Raw,
    SplitAdjusted,
    SplitDividendAdjusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquitySessionKind {
    PreMarket,
    Regular,
    AfterHours,
    Holiday,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquitySupport {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquitySymbol {
    pub canonical: String,
    pub vendor_symbol: String,
    pub market: EquityMarket,
    pub exchange: EquityExchangeCode,
    pub timezone: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityQuote {
    pub symbol: EquitySymbol,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub ts_ms: u64,
    pub latency: EquityLatencyClass,
    pub adjustment_mode: EquityAdjustmentMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityBar {
    pub symbol: EquitySymbol,
    pub timeframe: String,
    pub ts_open_ms: u64,
    pub ts_close_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub latency: EquityLatencyClass,
    pub adjustment_mode: EquityAdjustmentMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquitySessionWindow {
    pub kind: EquitySessionKind,
    pub start_local: String,
    pub end_local: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquityMarketCalendar {
    pub market: EquityMarket,
    pub exchange: EquityExchangeCode,
    pub timezone: String,
    pub date: String,
    pub sessions: Vec<EquitySessionWindow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquitySplit {
    pub numerator: f64,
    pub denominator: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityDividend {
    pub cash_amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EquityCorporateAction {
    Split {
        symbol: EquitySymbol,
        effective_date: String,
        split: EquitySplit,
    },
    ReverseSplit {
        symbol: EquitySymbol,
        effective_date: String,
        split: EquitySplit,
    },
    Dividend {
        symbol: EquitySymbol,
        ex_date: String,
        dividend: EquityDividend,
    },
    SymbolChange {
        from: EquitySymbol,
        to: EquitySymbol,
        effective_date: String,
    },
    Delist {
        symbol: EquitySymbol,
        effective_date: String,
    },
}

pub fn validate_bar_timeframe(tf: &str) -> Result<(), UcelError> {
    let ok = ["1m", "5m", "15m", "1h", "1d", "1w"].contains(&tf);
    if !ok {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "unsupported equity timeframe",
        ));
    }
    Ok(())
}

pub fn quote_is_stale(quote: &EquityQuote, now_ms: u64, max_age_ms: u64) -> bool {
    now_ms.saturating_sub(quote.ts_ms) > max_age_ms
}

pub fn session_includes_local_time(calendar: &EquityMarketCalendar, hhmm: &str) -> bool {
    calendar
        .sessions
        .iter()
        .any(|s| s.start_local.as_str() <= hhmm && hhmm <= s.end_local.as_str())
}

pub fn adjustment_mode_compatible(mode: EquityAdjustmentMode, has_actions: bool) -> bool {
    match mode {
        EquityAdjustmentMode::Raw => true,
        EquityAdjustmentMode::SplitAdjusted | EquityAdjustmentMode::SplitDividendAdjusted => {
            has_actions
        }
    }
}
