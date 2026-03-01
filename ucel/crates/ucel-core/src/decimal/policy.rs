use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

use super::guard::{DecimalGuard, DecimalGuardError};
use super::tick_step::{QuantizeMode, StepSize, TickSize, TickStepError};

#[derive(Debug, Clone)]
pub struct DecimalPolicy {
    pub max_scale: u32,
    pub max_abs: Option<Decimal>,
    pub price_rounding: RoundingStrategy,
    pub qty_rounding: RoundingStrategy,
    pub allow_negative: bool,
    pub allow_zero: bool,
}

impl Default for DecimalPolicy {
    fn default() -> Self {
        Self {
            max_scale: 18,
            max_abs: None,
            // "round half up" equivalent:
            price_rounding: RoundingStrategy::MidpointAwayFromZero,
            // conservative for quantities:
            qty_rounding: RoundingStrategy::ToZero,
            allow_negative: false,
            allow_zero: false,
        }
    }
}

impl DecimalPolicy {
    // Value-class policies (SSOT)
    pub fn for_execution_strict() -> Self {
        let mut p = Self::default();
        p.max_abs = Some(Decimal::from_str("1000000000000000000").expect("valid decimal")); // 1e18
        p
    }

    pub fn for_balance() -> Self {
        let mut p = Self::default();
        p.allow_zero = true;
        p.max_abs = Some(Decimal::from_str("1000000000000000000000000").expect("valid decimal")); // 1e24
        p
    }

    pub fn for_observation_relaxed() -> Self {
        let mut p = Self::default();
        p.allow_zero = true;
        p.allow_negative = true;
        p.max_abs = Some(Decimal::from_str("1000000000000000000000000").expect("valid decimal")); // 1e24
        p
    }

    pub fn guard(&self) -> DecimalGuard {
        DecimalGuard {
            max_scale: self.max_scale,
            max_abs: self.max_abs,
            allow_negative: self.allow_negative,
            allow_zero: self.allow_zero,
        }
    }

    pub fn round_price(&self, v: Decimal, scale: u32) -> Result<Decimal, DecimalGuardError> {
        let v = self.guard().validate(v)?;
        Ok(v.round_dp_with_strategy(scale, self.price_rounding))
    }

    pub fn round_qty(&self, v: Decimal, scale: u32) -> Result<Decimal, DecimalGuardError> {
        let v = self.guard().validate(v)?;
        Ok(v.round_dp_with_strategy(scale, self.qty_rounding))
    }

    pub fn validate_price_tick(
        &self,
        v: Decimal,
        tick: TickSize,
    ) -> Result<(), DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        tick.validate(v)?;
        Ok(())
    }

    pub fn validate_qty_step(&self, v: Decimal, step: StepSize) -> Result<(), DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        step.validate(v)?;
        Ok(())
    }

    pub fn quantize_price(
        &self,
        v: Decimal,
        tick: TickSize,
        mode: QuantizeMode,
    ) -> Result<Decimal, DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        Ok(tick.quantize(v, mode)?)
    }

    pub fn quantize_qty(
        &self,
        v: Decimal,
        step: StepSize,
        mode: QuantizeMode,
    ) -> Result<Decimal, DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        Ok(step.quantize(v, mode)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecimalPolicyError {
    #[error("decimal guard error: {0}")]
    Guard(#[from] DecimalGuardError),
    #[error("tick/step error: {0}")]
    TickStep(#[from] TickStepError),
}
