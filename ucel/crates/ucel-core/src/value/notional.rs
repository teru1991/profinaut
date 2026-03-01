use rust_decimal::Decimal;

use crate::decimal::guard::DecimalGuardError;
use crate::decimal::policy::DecimalPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notional(Decimal);

impl Notional {
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    pub fn try_new(policy: &DecimalPolicy, v: Decimal) -> Result<Self, DecimalGuardError> {
        let v = policy.guard().validate(v)?;
        Ok(Self(v))
    }
}
