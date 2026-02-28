UCEL Decimal Policy (A1) — Core Policy / Guard / Tick-Step (SSOT)

目的
	•	Decimal 導入だけでは不足するため、UCEL が 丸め規約・比較規約・tick/step 適用・不正値拒否を責務化し、発注/計算事故を防ぐ。
	•	本書は 実装の正本。Rust 側の追加ファイル雛形（全コード）を含む。

運用原則（固定）
	1.	生 Decimal を境界で禁止：発注・外部入出力は Newtype（Price/Qty 等）または Guard 済み Decimal へ。
	2.	丸め規約は用途別に固定（Price / Qty / Notional）。勝手に round しない。
	3.	tick/step は必ず適用：検証（validate）と量子化（quantize）を分離し、side別の安全丸めを提供。
	4.	不正値は即拒否：負値・0禁止（必要に応じて）・scale/桁制限・tick違反はエラー。

⸻

Rust 雛形（新規ファイル：ucel/crates/ucel-core/src/decimal/*）

NOTE: このタスクは docs-only。ここに載せたコードを後続のコード反映タスクで実ファイルへ配置する。

1) decimal/mod.rs

//! UCEL Decimal policy SSOT.
//! This module centralizes rounding/comparison/tick-step/guard rules.

pub mod policy;
pub mod guard;
pub mod tick_step;

// Optional: serde hooks (A2 task will add serde.rs and re-export here)

2) decimal/policy.rs

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

use super::guard::{DecimalGuard, DecimalGuardError};
use super::tick_step::{QuantizeMode, TickStepError, TickSize, StepSize};

/// Fixed policies used across UCEL.
/// Keep policy choices SSOT here.
#[derive(Debug, Clone)]
pub struct DecimalPolicy {
    /// Max allowed scale (fraction digits) after normalization.
    pub max_scale: u32,
    /// Optional max absolute magnitude (digits overflow guard).
    pub max_abs: Option<Decimal>,

    /// Rounding strategy for price and qty.
    pub price_rounding: RoundingStrategy,
    pub qty_rounding: RoundingStrategy,

    /// Whether to allow negative values. For trading values usually false.
    pub allow_negative: bool,
    /// Whether to allow zero values.
    pub allow_zero: bool,
}

impl Default for DecimalPolicy {
    fn default() -> Self {
        Self {
            max_scale: 18,
            max_abs: None,
            // NOTE: "round half up" equivalent is MidpointAwayFromZero in rust_decimal.
            // We fix it here; do not change without a migration plan.
            price_rounding: RoundingStrategy::MidpointAwayFromZero,
            // Qty should typically be conservative for safety; but the quantize mode will be chosen by caller.
            qty_rounding: RoundingStrategy::ToZero,
            allow_negative: false,
            allow_zero: false,
        }
    }
}

impl DecimalPolicy {
    pub fn guard(&self) -> DecimalGuard {
        DecimalGuard {
            max_scale: self.max_scale,
            max_abs: self.max_abs,
            allow_negative: self.allow_negative,
            allow_zero: self.allow_zero,
        }
    }

    /// Normalize scale and apply rounding strategy for a given "value class".
    pub fn round_price(&self, v: Decimal, scale: u32) -> Result<Decimal, DecimalGuardError> {
        let g = self.guard();
        let v = g.validate(v)?;
        Ok(v.round_dp_with_strategy(scale, self.price_rounding))
    }

    pub fn round_qty(&self, v: Decimal, scale: u32) -> Result<Decimal, DecimalGuardError> {
        let g = self.guard();
        let v = g.validate(v)?;
        Ok(v.round_dp_with_strategy(scale, self.qty_rounding))
    }

    /// Validate + tick quantize for price.
    pub fn quantize_price(
        &self,
        v: Decimal,
        tick: TickSize,
        mode: QuantizeMode,
    ) -> Result<Decimal, DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        Ok(tick.quantize(v, mode)?)
    }

    /// Validate + step quantize for qty.
    pub fn quantize_qty(
        &self,
        v: Decimal,
        step: StepSize,
        mode: QuantizeMode,
    ) -> Result<Decimal, DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        Ok(step.quantize(v, mode)?)
    }

    /// Strict check (no quantize) for price tick conformance.
    pub fn validate_price_tick(&self, v: Decimal, tick: TickSize) -> Result<(), DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        tick.validate(v)?;
        Ok(())
    }

    /// Strict check (no quantize) for qty step conformance.
    pub fn validate_qty_step(&self, v: Decimal, step: StepSize) -> Result<(), DecimalPolicyError> {
        let v = self.guard().validate(v)?;
        step.validate(v)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecimalPolicyError {
    #[error("decimal guard error: {0}")]
    Guard(#[from] DecimalGuardError),
    #[error("tick/step error: {0}")]
    TickStep(#[from] TickStepError),
}

3) decimal/guard.rs

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
        // Negative guard
        if !self.allow_negative && v.is_sign_negative() {
            return Err(DecimalGuardError::NegativeNotAllowed { value: v });
        }

        // Zero guard
        if !self.allow_zero && v.is_zero() {
            return Err(DecimalGuardError::ZeroNotAllowed);
        }

        // Scale guard
        let scale = v.scale();
        if scale > self.max_scale {
            return Err(DecimalGuardError::ScaleExceeded {
                scale,
                max_scale: self.max_scale,
            });
        }

        // Magnitude guard
        if let Some(max_abs) = self.max_abs {
            if v.abs() > max_abs {
                return Err(DecimalGuardError::MagnitudeExceeded {
                    value: v,
                    max_abs,
                });
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

4) decimal/tick_step.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizeMode {
    /// floor toward -inf
    Floor,
    /// ceil toward +inf
    Ceil,
    /// round to nearest tick/step (tie breaks away from zero)
    Nearest,
    /// truncate toward zero
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
    // v must be a multiple of unit (exact)
    let q = v / unit;
    if !q.fract().is_zero() {
        return Err(TickStepError::NotAMultiple { kind, value: v, unit });
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

    // NOTE: rust_decimal has floor/ceil/round hooks, but for ratio we implement simply.
    let q2 = match mode {
        QuantizeMode::Floor => q.floor(),
        QuantizeMode::Ceil => q.ceil(),
        QuantizeMode::ToZero => q.trunc(),
        QuantizeMode::Nearest => {
            // tie break away from zero
            let frac = q.fract().abs();
            if frac == Decimal::new(5, 1) {
                if q.is_sign_negative() { q.floor() } else { q.ceil() }
            } else {
                q.round()
            }
        }
    };

    let out = q2 * unit;
    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum TickStepError {
    #[error("{kind} unit is invalid (must be > 0)")]
    UnitInvalid { kind: &'static str },
    #[error("{kind} violation: value={value} is not a multiple of unit={unit}")]
    NotAMultiple { kind: &'static str, value: Decimal, unit: Decimal },
}
