use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizeMode {
    Floor,
    Ceil,
    Nearest,
    ToZero,
}

#[derive(Debug, Clone, Copy)]
pub struct TickSize(pub Decimal);

#[derive(Debug, Clone, Copy)]
pub struct StepSize(pub Decimal);

impl TickSize {
    pub fn validate(&self, v: Decimal) -> Result<(), TickStepError> {
        validate_multiple(v, self.0, "tick")
    }

    pub fn quantize(&self, v: Decimal, mode: QuantizeMode) -> Result<Decimal, TickStepError> {
        quantize_multiple(v, self.0, mode, "tick")
    }
}

impl StepSize {
    pub fn validate(&self, v: Decimal) -> Result<(), TickStepError> {
        validate_multiple(v, self.0, "step")
    }

    pub fn quantize(&self, v: Decimal, mode: QuantizeMode) -> Result<Decimal, TickStepError> {
        quantize_multiple(v, self.0, mode, "step")
    }
}

fn validate_multiple(v: Decimal, unit: Decimal, kind: &'static str) -> Result<(), TickStepError> {
    if unit <= Decimal::ZERO {
        return Err(TickStepError::UnitInvalid { kind });
    }
    let q = v / unit;
    if !q.fract().is_zero() {
        return Err(TickStepError::NotAMultiple {
            kind,
            value: v,
            unit,
        });
    }
    Ok(())
}

fn quantize_multiple(
    v: Decimal,
    unit: Decimal,
    mode: QuantizeMode,
    kind: &'static str,
) -> Result<Decimal, TickStepError> {
    if unit <= Decimal::ZERO {
        return Err(TickStepError::UnitInvalid { kind });
    }
    let q = v / unit;
    let q2 = match mode {
        QuantizeMode::Floor => q.floor(),
        QuantizeMode::Ceil => q.ceil(),
        QuantizeMode::ToZero => q.trunc(),
        QuantizeMode::Nearest => {
            // tie-break away from zero for .5
            let frac = q.fract().abs();
            if frac == Decimal::new(5, 1) {
                if q.is_sign_negative() {
                    q.floor()
                } else {
                    q.ceil()
                }
            } else {
                q.round()
            }
        }
    };
    Ok(q2 * unit)
}

#[derive(Debug, thiserror::Error)]
pub enum TickStepError {
    #[error("{kind} unit is invalid (must be > 0)")]
    UnitInvalid { kind: &'static str },
    #[error("{kind} violation: value={value} is not a multiple of unit={unit}")]
    NotAMultiple {
        kind: &'static str,
        value: Decimal,
        unit: Decimal,
    },
}
