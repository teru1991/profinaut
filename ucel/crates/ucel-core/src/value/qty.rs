use rust_decimal::Decimal;

use crate::decimal::policy::{DecimalPolicy, DecimalPolicyError};
use crate::decimal::tick_step::QuantizeMode;
use crate::value::StepSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qty(Decimal);

impl Qty {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    pub fn try_new_strict(
        policy: &DecimalPolicy,
        v: Decimal,
        step: StepSize,
    ) -> Result<Self, DecimalPolicyError> {
        policy.validate_qty_step(v, step.to_core())?;
        Ok(Self(v))
    }

    pub fn try_new_quantized(
        policy: &DecimalPolicy,
        v: Decimal,
        step: StepSize,
        mode: QuantizeMode,
    ) -> Result<Self, DecimalPolicyError> {
        let v2 = policy.quantize_qty(v, step.to_core(), mode)?;
        Ok(Self(v2))
    }
}
