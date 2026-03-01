use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct DecimalGuard {
    pub max_scale: u32,
    pub max_abs: Option<Decimal>,
    pub allow_negative: bool,
    pub allow_zero: bool,
}

impl DecimalGuard {
    pub fn validate(&self, v: Decimal) -> Result<Decimal, DecimalGuardError> {
        if !self.allow_negative && v.is_sign_negative() {
            return Err(DecimalGuardError::NegativeNotAllowed { value: v });
        }
        if !self.allow_zero && v.is_zero() {
            return Err(DecimalGuardError::ZeroNotAllowed);
        }
        let scale = v.scale();
        if scale > self.max_scale {
            return Err(DecimalGuardError::ScaleExceeded {
                scale,
                max_scale: self.max_scale,
            });
        }
        if let Some(max_abs) = self.max_abs {
            if v.abs() > max_abs {
                return Err(DecimalGuardError::MagnitudeExceeded { value: v, max_abs });
            }
        }
        Ok(v)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecimalGuardError {
    #[error("negative decimal is not allowed: {value}")]
    NegativeNotAllowed { value: Decimal },
    #[error("zero is not allowed")]
    ZeroNotAllowed,
    #[error("scale exceeded: scale={scale}, max_scale={max_scale}")]
    ScaleExceeded { scale: u32, max_scale: u32 },
    #[error("magnitude exceeded: value={value}, max_abs={max_abs}")]
    MagnitudeExceeded { value: Decimal, max_abs: Decimal },
}
