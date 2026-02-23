use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::SystemTime;

pub type InstrumentMeta = BTreeMap<String, serde_json::Value>;
pub const SYMBOL_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exchange {
    Binance,
    BinanceUsdm,
    BinanceCoinm,
    BinanceOptions,
    Bitbank,
    Bitflyer,
    Bitget,
    Bitmex,
    Bittrade,
    Bybit,
    Coinbase,
    Coincheck,
    Deribit,
    Gmocoin,
    Htx,
    Kraken,
    Okx,
    Sbivc,
    Upbit,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketType {
    Spot,
    LinearPerpetual,
    InversePerpetual,
    Option,
    Delivery,
    Margin,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionRight {
    Call,
    Put,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolStatus {
    Trading,
    Suspended,
    PreMarket,
    PostMarket,
    Delisted,
    Break,
    Maintenance,
    Auction,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstrumentId {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub raw_symbol: String,
    pub expiry: Option<String>,
    pub strike: Option<Decimal>,
    pub option_right: Option<OptionRight>,
    pub contract_size: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardizedInstrument {
    pub id: InstrumentId,
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub base: String,
    pub quote: String,
    pub raw_symbol: String,
    pub status: SymbolStatus,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub min_order_qty: Option<Decimal>,
    pub max_order_qty: Option<Decimal>,
    pub min_notional: Option<Decimal>,
    pub price_precision: Option<u32>,
    pub qty_precision: Option<u32>,
    pub contract_size: Option<Decimal>,
    pub meta: InstrumentMeta,
    pub ts_recv: SystemTime,
    pub ts_event: Option<SystemTime>,
    pub schema_version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotSource {
    Rest,
    Ws,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotOrigin {
    pub source: SnapshotSource,
    pub restored: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub ts_recv: SystemTime,
    pub instruments: Vec<StandardizedInstrument>,
    pub origin: SnapshotOrigin,
}

impl Snapshot {
    pub fn new_rest(instruments: Vec<StandardizedInstrument>) -> Self {
        Self {
            snapshot_id: uuid::Uuid::new_v4().to_string(),
            ts_recv: SystemTime::now(),
            instruments,
            origin: SnapshotOrigin {
                source: SnapshotSource::Rest,
                restored: false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaCompatibility {
    Compatible,
    RequiresUpgrade,
    UnsupportedFuture,
}

pub fn check_schema_compatibility(schema_version: u16) -> SchemaCompatibility {
    use std::cmp::Ordering::*;
    match schema_version.cmp(&SYMBOL_SCHEMA_VERSION) {
        Equal => SchemaCompatibility::Compatible,
        Less => SchemaCompatibility::RequiresUpgrade,
        Greater => SchemaCompatibility::UnsupportedFuture,
    }
}

pub fn stash_unknown_in_meta(
    meta: &mut InstrumentMeta,
    key: impl Into<String>,
    value: serde_json::Value,
) {
    meta.insert(key.into(), value);
}

pub fn cmp_decimal(a: Decimal, b: Decimal) -> std::cmp::Ordering {
    normalize_decimal(a).cmp(&normalize_decimal(b))
}

pub fn normalize_decimal(x: Decimal) -> Decimal {
    x.normalize()
}

pub fn round_price(value: Decimal, precision: u32) -> Decimal {
    value.round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero)
}

pub fn round_qty(value: Decimal, precision: u32) -> Decimal {
    value.round_dp_with_strategy(precision, RoundingStrategy::ToZero)
}

pub fn format_decimal(value: Decimal) -> String {
    normalize_decimal(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromStr;

    #[test]
    fn precision_helpers_work() {
        let a = Decimal::from_str("1.2300").unwrap();
        let b = Decimal::from_str("1.23").unwrap();
        assert_eq!(cmp_decimal(a, b), std::cmp::Ordering::Equal);
        assert_eq!(normalize_decimal(a), b);
        assert_eq!(
            round_price(Decimal::from_str("1.235").unwrap(), 2).to_string(),
            "1.24"
        );
        assert_eq!(
            round_qty(Decimal::from_str("1.239").unwrap(), 2).to_string(),
            "1.23"
        );
        assert_eq!(format_decimal(a), "1.23");
    }

    #[test]
    fn schema_compatibility_and_meta_helpers_work() {
        assert_eq!(
            check_schema_compatibility(SYMBOL_SCHEMA_VERSION),
            SchemaCompatibility::Compatible
        );
        assert_eq!(
            check_schema_compatibility(SYMBOL_SCHEMA_VERSION.saturating_sub(1)),
            SchemaCompatibility::RequiresUpgrade
        );
        assert_eq!(
            check_schema_compatibility(SYMBOL_SCHEMA_VERSION + 1),
            SchemaCompatibility::UnsupportedFuture
        );

        let mut meta = InstrumentMeta::new();
        stash_unknown_in_meta(&mut meta, "unknown_status", serde_json::json!("halted"));
        assert_eq!(
            meta.get("unknown_status"),
            Some(&serde_json::json!("halted"))
        );
    }
}
