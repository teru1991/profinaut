use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::SystemTime;
use ucel_core::decimal::{
    CoreStepSize, CoreTickSize, DecimalPolicy, DecimalPolicyError, QuantizeMode,
};
use ucel_core::Decimal;

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

fn policy_relaxed() -> DecimalPolicy {
    let p = DecimalPolicy::for_observation_relaxed();
    // symbol層は “丸め/量子化” の補助。execution gate には使わない。
    p
}

pub fn round_price(value: Decimal, precision: u32) -> Decimal {
    policy_relaxed()
        .round_price(value, precision)
        .expect("symbol-layer rounding must not fail under relaxed policy")
}

pub fn round_qty(value: Decimal, precision: u32) -> Decimal {
    policy_relaxed()
        .round_qty(value, precision)
        .expect("symbol-layer rounding must not fail under relaxed policy")
}

// Tick/Step SSOT helpers (additive API)
pub fn validate_price_tick(price: Decimal, tick_size: Decimal) -> Result<(), DecimalPolicyError> {
    let p = policy_relaxed();
    p.validate_price_tick(price, CoreTickSize(tick_size))
}

pub fn validate_qty_step(qty: Decimal, step_size: Decimal) -> Result<(), DecimalPolicyError> {
    let p = policy_relaxed();
    p.validate_qty_step(qty, CoreStepSize(step_size))
}

pub fn quantize_price(
    price: Decimal,
    tick_size: Decimal,
    mode: QuantizeMode,
) -> Result<Decimal, DecimalPolicyError> {
    let p = policy_relaxed();
    p.quantize_price(price, CoreTickSize(tick_size), mode)
}

pub fn quantize_qty(
    qty: Decimal,
    step_size: Decimal,
    mode: QuantizeMode,
) -> Result<Decimal, DecimalPolicyError> {
    let p = policy_relaxed();
    p.quantize_qty(qty, CoreStepSize(step_size), mode)
}

pub fn format_decimal(value: Decimal) -> String {
    normalize_decimal(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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

        assert!(validate_price_tick(
            Decimal::from_str("1.23").unwrap(),
            Decimal::from_str("0.01").unwrap()
        )
        .is_ok());
        assert!(validate_qty_step(
            Decimal::from_str("0.123").unwrap(),
            Decimal::from_str("0.001").unwrap()
        )
        .is_ok());
        assert_eq!(
            quantize_price(
                Decimal::from_str("1.234").unwrap(),
                Decimal::from_str("0.01").unwrap(),
                QuantizeMode::Floor,
            )
            .unwrap()
            .to_string(),
            "1.23"
        );
        assert_eq!(
            quantize_qty(
                Decimal::from_str("0.1239").unwrap(),
                Decimal::from_str("0.001").unwrap(),
                QuantizeMode::ToZero,
            )
            .unwrap()
            .to_string(),
            "0.123"
        );
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
