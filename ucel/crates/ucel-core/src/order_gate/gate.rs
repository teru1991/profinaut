use crate::decimal::policy::{DecimalPolicy, DecimalPolicyError};
use crate::decimal::tick_step::QuantizeMode;
use crate::value::{Price, Qty, StepSize, TickSize};
use crate::{Decimal, Side};

#[derive(Debug, Clone)]
pub struct OrderGate {
    policy: DecimalPolicy,
}

impl Default for OrderGate {
    fn default() -> Self {
        Self {
            policy: DecimalPolicy::for_execution_strict(),
        }
    }
}

impl OrderGate {
    pub fn new(policy: DecimalPolicy) -> Self {
        Self { policy }
    }

    pub fn validate_limit(
        &self,
        price: Decimal,
        qty: Decimal,
        tick: TickSize,
        step: StepSize,
    ) -> Result<(Price, Qty), OrderGateError> {
        self.policy.validate_price_tick(price, tick.to_core())?;
        self.policy.validate_qty_step(qty, step.to_core())?;
        Ok((
            Price::try_new_strict(&self.policy, price, tick)?,
            Qty::try_new_strict(&self.policy, qty, step)?,
        ))
    }

    pub fn quantize_limit(
        &self,
        side: Side,
        price: Decimal,
        qty: Decimal,
        tick: TickSize,
        step: StepSize,
        price_mode: QuantizeMode,
        qty_mode: QuantizeMode,
    ) -> Result<(Price, Qty), OrderGateError> {
        let _ = side;
        let p = Price::try_new_quantized(&self.policy, price, tick, price_mode)?;
        let q = Qty::try_new_quantized(&self.policy, qty, step, qty_mode)?;
        self.policy
            .validate_price_tick(p.as_decimal(), tick.to_core())?;
        self.policy
            .validate_qty_step(q.as_decimal(), step.to_core())?;
        Ok((p, q))
    }

    pub fn recommended_modes(side: Side) -> (QuantizeMode, QuantizeMode) {
        let qty_mode = QuantizeMode::ToZero;
        let price_mode = match side {
            Side::Buy => QuantizeMode::Ceil,
            Side::Sell => QuantizeMode::Floor,
            _ => QuantizeMode::Nearest,
        };
        (price_mode, qty_mode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OrderGateError {
    #[error("decimal policy error: {0}")]
    Policy(#[from] DecimalPolicyError),
}
