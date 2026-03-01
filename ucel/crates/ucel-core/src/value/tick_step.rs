use rust_decimal::Decimal;

use crate::decimal::tick_step::{StepSize as CoreStepSize, TickSize as CoreTickSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickSize(pub Decimal);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepSize(pub Decimal);

impl TickSize {
    pub fn to_core(self) -> CoreTickSize {
        CoreTickSize(self.0)
    }
}
impl StepSize {
    pub fn to_core(self) -> CoreStepSize {
        CoreStepSize(self.0)
    }
}
