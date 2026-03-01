use rust_decimal::Decimal;

use crate::decimal::policy::{DecimalPolicy, DecimalPolicyError};
use crate::decimal::tick_step::QuantizeMode;
use crate::value::TickSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price(Decimal);

impl Price {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    pub fn try_new_strict(
        policy: &DecimalPolicy,
        v: Decimal,
        tick: TickSize,
    ) -> Result<Self, DecimalPolicyError> {
        policy.validate_price_tick(v, tick.to_core())?;
        Ok(Self(v))
    }

    pub fn try_new_quantized(
        policy: &DecimalPolicy,
        v: Decimal,
        tick: TickSize,
        mode: QuantizeMode,
    ) -> Result<Self, DecimalPolicyError> {
        let v2 = policy.quantize_price(v, tick.to_core(), mode)?;
        Ok(Self(v2))
    }
}
