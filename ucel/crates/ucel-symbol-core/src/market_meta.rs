use crate::{
    normalize_decimal, validate_price_tick, validate_qty_step, Exchange, InstrumentMeta,
    MarketType, SnapshotOrigin, StandardizedInstrument,
};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub const MARKET_META_SCHEMA_VERSION: u16 = 1;

fn default_market_meta_schema_version() -> u16 {
    MARKET_META_SCHEMA_VERSION
}

fn default_snapshot_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_ts_recv() -> SystemTime {
    SystemTime::now()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MarketMetaId {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub raw_symbol: String,
}

impl MarketMetaId {
    pub fn new(exchange: Exchange, market_type: MarketType, raw_symbol: impl Into<String>) -> Self {
        Self {
            exchange,
            market_type,
            raw_symbol: raw_symbol.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TickStepRounding {
    Down,
    Up,
    Nearest,
}

impl Default for TickStepRounding {
    fn default() -> Self {
        Self::Nearest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum MarketMetaError {
    #[error("invalid market meta: {0}")]
    InvalidMeta(&'static str),
    #[error("invalid value: {0}")]
    InvalidValue(&'static str),
    #[error("constraint violation: {0}")]
    Constraint(&'static str),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMeta {
    pub id: MarketMetaId,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub quote: Option<String>,
    pub tick_size: Decimal,
    pub step_size: Decimal,
    #[serde(default)]
    pub min_qty: Option<Decimal>,
    #[serde(default)]
    pub max_qty: Option<Decimal>,
    #[serde(default)]
    pub min_notional: Option<Decimal>,
    #[serde(default)]
    pub price_precision: Option<u32>,
    #[serde(default)]
    pub qty_precision: Option<u32>,
    #[serde(default)]
    pub contract_size: Option<Decimal>,
    #[serde(default)]
    pub meta: InstrumentMeta,
    #[serde(default = "default_market_meta_schema_version")]
    pub schema_version: u16,
}

impl MarketMeta {
    /// Symbol層は観測層のため、形式チェックのみを提供する。
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.tick_size <= Decimal::ZERO {
            return Err(format!("invalid tick_size={}", self.tick_size));
        }
        if self.step_size <= Decimal::ZERO {
            return Err(format!("invalid step_size={}", self.step_size));
        }
        if let Some(q) = self.min_qty {
            if q <= Decimal::ZERO {
                return Err(format!("invalid min_qty={}", q));
            }
        }
        if let Some(n) = self.min_notional {
            if n <= Decimal::ZERO {
                return Err(format!("invalid min_notional={}", n));
            }
        }
        Ok(())
    }

    pub fn validate_price_tick_relaxed(
        &self,
        price: Decimal,
    ) -> Result<(), ucel_core::decimal::DecimalPolicyError> {
        validate_price_tick(price, self.tick_size)
    }

    pub fn validate_qty_step_relaxed(
        &self,
        qty: Decimal,
    ) -> Result<(), ucel_core::decimal::DecimalPolicyError> {
        validate_qty_step(qty, self.step_size)
    }

    pub fn new(id: MarketMetaId, tick_size: Decimal, step_size: Decimal) -> Self {
        Self {
            id,
            base: None,
            quote: None,
            tick_size,
            step_size,
            min_qty: None,
            max_qty: None,
            min_notional: None,
            price_precision: None,
            qty_precision: None,
            contract_size: None,
            meta: InstrumentMeta::new(),
            schema_version: MARKET_META_SCHEMA_VERSION,
        }
    }

    pub fn validate_meta(&self) -> Result<(), MarketMetaError> {
        if self.schema_version != MARKET_META_SCHEMA_VERSION {
            return Err(MarketMetaError::InvalidMeta("schema_version_mismatch"));
        }
        if self.tick_size <= Decimal::ZERO {
            return Err(MarketMetaError::InvalidMeta("tick_size_must_be_positive"));
        }
        if self.step_size <= Decimal::ZERO {
            return Err(MarketMetaError::InvalidMeta("step_size_must_be_positive"));
        }

        if let Some(min_qty) = self.min_qty {
            if min_qty < Decimal::ZERO {
                return Err(MarketMetaError::InvalidMeta("min_qty_must_be_non_negative"));
            }
            if min_qty > Decimal::ZERO && !is_multiple_of(min_qty, self.step_size) {
                return Err(MarketMetaError::InvalidMeta("min_qty_not_multiple_of_step"));
            }
        }
        if let Some(max_qty) = self.max_qty {
            if max_qty < Decimal::ZERO {
                return Err(MarketMetaError::InvalidMeta("max_qty_must_be_non_negative"));
            }
            if max_qty > Decimal::ZERO && !is_multiple_of(max_qty, self.step_size) {
                return Err(MarketMetaError::InvalidMeta("max_qty_not_multiple_of_step"));
            }
        }
        if let (Some(min_qty), Some(max_qty)) = (self.min_qty, self.max_qty) {
            if min_qty > max_qty {
                return Err(MarketMetaError::InvalidMeta("min_qty_greater_than_max_qty"));
            }
        }

        if let Some(min_notional) = self.min_notional {
            if min_notional < Decimal::ZERO {
                return Err(MarketMetaError::InvalidMeta(
                    "min_notional_must_be_non_negative",
                ));
            }
        }

        if let Some(cs) = self.contract_size {
            if cs <= Decimal::ZERO {
                return Err(MarketMetaError::InvalidMeta(
                    "contract_size_must_be_positive_if_present",
                ));
            }
        }

        Ok(())
    }

    pub fn is_price_tick_aligned(&self, price: Decimal) -> bool {
        is_multiple_of(price, self.tick_size)
    }

    pub fn is_qty_step_aligned(&self, qty: Decimal) -> bool {
        is_multiple_of(qty, self.step_size)
    }

    pub fn apply_tick(
        &self,
        price: Decimal,
        rounding: TickStepRounding,
    ) -> Result<Decimal, MarketMetaError> {
        self.validate_meta()?;
        ensure_non_negative(price, "price_must_be_non_negative")?;
        Ok(quantize(price, self.tick_size, rounding))
    }

    pub fn apply_step(
        &self,
        qty: Decimal,
        rounding: TickStepRounding,
    ) -> Result<Decimal, MarketMetaError> {
        self.validate_meta()?;
        ensure_non_negative(qty, "qty_must_be_non_negative")?;
        Ok(quantize(qty, self.step_size, rounding))
    }

    pub fn apply_tick_for_side(
        &self,
        price: Decimal,
        side: OrderSide,
    ) -> Result<Decimal, MarketMetaError> {
        match side {
            OrderSide::Buy => self.apply_tick(price, TickStepRounding::Down),
            OrderSide::Sell => self.apply_tick(price, TickStepRounding::Up),
        }
    }

    pub fn apply_step_safe(&self, qty: Decimal) -> Result<Decimal, MarketMetaError> {
        self.apply_step(qty, TickStepRounding::Down)
    }

    pub fn compute_notional(
        &self,
        price: Decimal,
        qty: Decimal,
    ) -> Result<Decimal, MarketMetaError> {
        self.validate_meta()?;
        ensure_non_negative(price, "price_must_be_non_negative")?;
        ensure_non_negative(qty, "qty_must_be_non_negative")?;

        let cs = self.contract_size.unwrap_or(Decimal::ONE);
        Ok(normalize_decimal(price * qty * cs))
    }

    pub fn check_min_notional(&self, price: Decimal, qty: Decimal) -> Result<(), MarketMetaError> {
        if let Some(min_notional) = self.min_notional {
            let notional = self.compute_notional(price, qty)?;
            if notional < min_notional {
                return Err(MarketMetaError::Constraint("min_notional_not_satisfied"));
            }
        }
        Ok(())
    }

    pub fn validate_order(&self, price: Decimal, qty: Decimal) -> Result<(), MarketMetaError> {
        self.validate_meta()?;
        ensure_positive(price, "price_must_be_positive")?;
        ensure_positive(qty, "qty_must_be_positive")?;

        if !self.is_price_tick_aligned(price) {
            return Err(MarketMetaError::Constraint("price_not_tick_aligned"));
        }
        if !self.is_qty_step_aligned(qty) {
            return Err(MarketMetaError::Constraint("qty_not_step_aligned"));
        }

        if let Some(min_qty) = self.min_qty {
            if qty < min_qty {
                return Err(MarketMetaError::Constraint("qty_below_min_qty"));
            }
        }
        if let Some(max_qty) = self.max_qty {
            if qty > max_qty {
                return Err(MarketMetaError::Constraint("qty_above_max_qty"));
            }
        }

        self.check_min_notional(price, qty)?;
        Ok(())
    }

    pub fn normalize_order(
        &self,
        price: Decimal,
        qty: Decimal,
        side: OrderSide,
    ) -> Result<(Decimal, Decimal), MarketMetaError> {
        let p = self.apply_tick_for_side(price, side)?;
        let q = self.apply_step_safe(qty)?;
        self.validate_order(p, q)?;
        Ok((p, q))
    }

    pub fn from_standardized_instrument(ins: &crate::StandardizedInstrument) -> Self {
        let id = MarketMetaId {
            exchange: ins.exchange.clone(),
            market_type: ins.market_type.clone(),
            raw_symbol: ins.raw_symbol.clone(),
        };
        Self {
            id,
            base: Some(ins.base.clone()),
            quote: Some(ins.quote.clone()),
            tick_size: ins.tick_size,
            step_size: ins.lot_size,
            min_qty: ins.min_order_qty,
            max_qty: ins.max_order_qty,
            min_notional: ins.min_notional,
            price_precision: ins.price_precision,
            qty_precision: ins.qty_precision,
            contract_size: ins.contract_size,
            meta: ins.meta.clone(),
            schema_version: MARKET_META_SCHEMA_VERSION,
        }
    }
}

impl From<&StandardizedInstrument> for MarketMeta {
    fn from(si: &StandardizedInstrument) -> Self {
        let mut mm = Self::new(
            MarketMetaId::new(
                si.exchange.clone(),
                si.market_type.clone(),
                si.raw_symbol.clone(),
            ),
            si.tick_size,
            si.lot_size,
        );
        mm.base = Some(si.base.clone());
        mm.quote = Some(si.quote.clone());
        mm.min_qty = si.min_order_qty;
        mm.max_qty = si.max_order_qty;
        mm.min_notional = si.min_notional;
        mm.price_precision = si.price_precision;
        mm.qty_precision = si.qty_precision;
        mm.contract_size = si.contract_size;
        mm.meta = si.meta.clone();
        mm
    }
}

impl From<StandardizedInstrument> for MarketMeta {
    fn from(si: StandardizedInstrument) -> Self {
        Self::from(&si)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMetaSnapshot {
    #[serde(default = "default_snapshot_id")]
    pub snapshot_id: String,
    #[serde(default = "default_ts_recv")]
    pub ts_recv: SystemTime,
    pub markets: Vec<MarketMeta>,
    #[serde(default)]
    pub origin: SnapshotOrigin,
}

impl MarketMetaSnapshot {
    pub fn new_rest(markets: Vec<MarketMeta>) -> Self {
        Self {
            snapshot_id: default_snapshot_id(),
            ts_recv: SystemTime::now(),
            markets,
            origin: SnapshotOrigin::default(),
        }
    }
}

fn ensure_non_negative(v: Decimal, code: &'static str) -> Result<(), MarketMetaError> {
    if v < Decimal::ZERO {
        return Err(MarketMetaError::InvalidValue(code));
    }
    Ok(())
}

fn ensure_positive(v: Decimal, code: &'static str) -> Result<(), MarketMetaError> {
    if v <= Decimal::ZERO {
        return Err(MarketMetaError::InvalidValue(code));
    }
    Ok(())
}

fn is_multiple_of(value: Decimal, step: Decimal) -> bool {
    if step == Decimal::ZERO {
        return false;
    }
    let v = normalize_decimal(value);
    let s = normalize_decimal(step);
    let rem = normalize_decimal(v % s);
    rem == Decimal::ZERO
}

fn quantize(value: Decimal, step: Decimal, rounding: TickStepRounding) -> Decimal {
    let v = normalize_decimal(value);
    let s = normalize_decimal(step);

    let q = normalize_decimal(v / s);
    let strategy = match rounding {
        TickStepRounding::Down => RoundingStrategy::ToZero,
        TickStepRounding::Up => RoundingStrategy::ToPositiveInfinity,
        TickStepRounding::Nearest => RoundingStrategy::MidpointAwayFromZero,
    };
    let q_i = q.round_dp_with_strategy(0, strategy);
    normalize_decimal(q_i * s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromStr;

    #[test]
    fn tick_step_quantize_works_and_is_deterministic() {
        let id = MarketMetaId::new(Exchange::Bitbank, MarketType::Spot, "BTC/JPY");
        let meta = MarketMeta {
            tick_size: Decimal::from_str("1").unwrap(),
            step_size: Decimal::from_str("0.0001").unwrap(),
            min_qty: Some(Decimal::from_str("0.0001").unwrap()),
            min_notional: Some(Decimal::from_str("500").unwrap()),
            ..MarketMeta::new(
                id,
                Decimal::from_str("1").unwrap(),
                Decimal::from_str("0.0001").unwrap(),
            )
        };
        meta.validate_meta().unwrap();

        let p = Decimal::from_str("123.9").unwrap();
        assert_eq!(
            meta.apply_tick(p, TickStepRounding::Down)
                .unwrap()
                .to_string(),
            "123"
        );
        assert_eq!(
            meta.apply_tick(p, TickStepRounding::Up)
                .unwrap()
                .to_string(),
            "124"
        );
        assert_eq!(
            meta.apply_tick(p, TickStepRounding::Nearest)
                .unwrap()
                .to_string(),
            "124"
        );

        let q = Decimal::from_str("0.123456").unwrap();
        assert_eq!(
            meta.apply_step(q, TickStepRounding::Down)
                .unwrap()
                .to_string(),
            "0.1234"
        );
        assert_eq!(
            meta.apply_step(q, TickStepRounding::Up)
                .unwrap()
                .to_string(),
            "0.1235"
        );
    }

    #[test]
    fn validate_order_rejects_misalignment_and_min_notional() {
        let id = MarketMetaId::new(Exchange::Bitbank, MarketType::Spot, "BTC/JPY");
        let meta = MarketMeta {
            tick_size: Decimal::from_str("1").unwrap(),
            step_size: Decimal::from_str("0.0001").unwrap(),
            min_qty: Some(Decimal::from_str("0.0001").unwrap()),
            min_notional: Some(Decimal::from_str("500").unwrap()),
            ..MarketMeta::new(
                id,
                Decimal::from_str("1").unwrap(),
                Decimal::from_str("0.0001").unwrap(),
            )
        };

        assert!(meta
            .validate_order(
                Decimal::from_str("10.5").unwrap(),
                Decimal::from_str("1").unwrap()
            )
            .is_err());

        assert!(meta
            .validate_order(
                Decimal::from_str("10").unwrap(),
                Decimal::from_str("0.00011").unwrap()
            )
            .is_err());

        assert!(meta
            .validate_order(
                Decimal::from_str("100").unwrap(),
                Decimal::from_str("0.0001").unwrap()
            )
            .is_err());

        let norm = meta.normalize_order(
            Decimal::from_str("100.9").unwrap(),
            Decimal::from_str("0.00019").unwrap(),
            OrderSide::Buy,
        );
        assert!(norm.is_err());
    }

    #[test]
    fn market_meta_is_derived_from_standardized_instrument() {
        let si = StandardizedInstrument {
            id: crate::InstrumentId {
                exchange: Exchange::Other("dummy".into()),
                market_type: MarketType::Spot,
                raw_symbol: "AAA/BBB".into(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Other("dummy".into()),
            market_type: MarketType::Spot,
            base: "AAA".into(),
            quote: "BBB".into(),
            raw_symbol: "AAA/BBB".into(),
            status: crate::SymbolStatus::Trading,
            tick_size: Decimal::from_str("0.01").unwrap(),
            lot_size: Decimal::from_str("0.001").unwrap(),
            min_order_qty: Some(Decimal::from_str("0.01").unwrap()),
            max_order_qty: None,
            min_notional: Some(Decimal::from_str("5").unwrap()),
            price_precision: None,
            qty_precision: None,
            contract_size: None,
            meta: crate::InstrumentMeta::new(),
            ts_recv: std::time::SystemTime::now(),
            ts_event: None,
            schema_version: crate::SYMBOL_SCHEMA_VERSION,
        };

        let mm = MarketMeta::from(&si);
        assert_eq!(mm.tick_size.to_string(), "0.01");
        assert_eq!(mm.step_size.to_string(), "0.001");
        assert_eq!(mm.min_qty.unwrap().to_string(), "0.01");
        assert_eq!(mm.min_notional.unwrap().to_string(), "5");
        assert!(mm.validate_basic().is_ok());
    }
}
