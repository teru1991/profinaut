use ucel_core::decimal::tick_step::QuantizeMode;
use ucel_core::order_gate::OrderGate;
use ucel_core::{Decimal, Side, StepSize, TickSize};

#[test]
fn order_gate_quantizes_and_validates() {
    let gate = OrderGate::default();
    let tick = TickSize(Decimal::from_str_exact("0.01").unwrap());
    let step = StepSize(Decimal::from_str_exact("0.001").unwrap());

    let raw_price = Decimal::from_str_exact("100.009").unwrap();
    let raw_qty = Decimal::from_str_exact("0.12345").unwrap();

    let (p, q) = gate
        .quantize_limit(
            Side::Buy,
            raw_price,
            raw_qty,
            tick,
            step,
            QuantizeMode::Floor,
            QuantizeMode::ToZero,
        )
        .unwrap();

    assert_eq!(p.as_decimal(), Decimal::from_str_exact("100.00").unwrap());
    assert_eq!(q.as_decimal(), Decimal::from_str_exact("0.123").unwrap());
}

#[test]
fn order_gate_rejects_tick_violation_in_strict_validate() {
    let gate = OrderGate::default();
    let tick = TickSize(Decimal::from_str_exact("0.01").unwrap());
    let step = StepSize(Decimal::from_str_exact("0.001").unwrap());

    let price = Decimal::from_str_exact("1.001").unwrap();
    let qty = Decimal::from_str_exact("0.010").unwrap();
    assert!(gate.validate_limit(price, qty, tick, step).is_err());
}
