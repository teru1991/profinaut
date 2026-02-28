UCEL Decimal Policy (C) — Tests Scaffold (SSOT)

新規テスト（正本・全コード）:
	•	ucel/crates/ucel-core/tests/decimal_policy_rounding.rs
	•	ucel/crates/ucel-core/tests/tick_step_quantize.rs
	•	ucel/crates/ucel-core/tests/decimal_guard_rejects_invalid.rs

1) decimal_policy_rounding.rs

use rust_decimal::Decimal;
use ucel_core::decimal::policy::DecimalPolicy;

#[test]
fn price_round_half_up_equivalent() {
    let p = DecimalPolicy::default();

    // 100.005 -> 100.01 when scale=2
    let v = Decimal::from_str_exact("100.005").unwrap();
    let out = p.round_price(v, 2).unwrap();
    assert_eq!(out, Decimal::from_str_exact("100.01").unwrap());

    // negative midpoint: -1.005 -> -1.01 (away from zero)
    let v = Decimal::from_str_exact("-1.005").unwrap();
    let err = p.round_price(v, 2).unwrap_err();
    // default policy disallows negative -> should fail guard
    let _ = err;
}

2) tick_step_quantize.rs

use rust_decimal::Decimal;
use ucel_core::decimal::policy::DecimalPolicy;
use ucel_core::decimal::tick_step::{QuantizeMode, TickSize, StepSize};

#[test]
fn quantize_price_by_tick() {
    let p = DecimalPolicy::default();
    let tick = TickSize(Decimal::from_str_exact("0.01").unwrap());

    let v = Decimal::from_str_exact("100.009").unwrap();
    let out = p.quantize_price(v, tick, QuantizeMode::Floor).unwrap();
    assert_eq!(out, Decimal::from_str_exact("100.00").unwrap());

    let v = Decimal::from_str_exact("100.005").unwrap();
    let out = p.quantize_price(v, tick, QuantizeMode::Nearest).unwrap();
    assert_eq!(out, Decimal::from_str_exact("100.01").unwrap());
}

#[test]
fn quantize_qty_by_step() {
    let p = DecimalPolicy::default();
    let step = StepSize(Decimal::from_str_exact("0.001").unwrap());

    let v = Decimal::from_str_exact("0.12345").unwrap();
    let out = p.quantize_qty(v, step, QuantizeMode::ToZero).unwrap();
    assert_eq!(out, Decimal::from_str_exact("0.123").unwrap());
}

3) decimal_guard_rejects_invalid.rs

use rust_decimal::Decimal;
use ucel_core::decimal::guard::DecimalGuard;

#[test]
fn reject_negative_and_zero_by_default_guard() {
    let g = DecimalGuard {
        max_scale: 18,
        max_abs: None,
        allow_negative: false,
        allow_zero: false,
    };

    assert!(g.validate(Decimal::from_str_exact("-1").unwrap()).is_err());
    assert!(g.validate(Decimal::ZERO).is_err());
}

#[test]
fn reject_scale_overflow() {
    let g = DecimalGuard {
        max_scale: 2,
        max_abs: None,
        allow_negative: true,
        allow_zero: true,
    };

    assert!(g.validate(Decimal::from_str_exact("1.001").unwrap()).is_err());
}
