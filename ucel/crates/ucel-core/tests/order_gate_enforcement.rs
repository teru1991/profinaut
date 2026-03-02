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

#[test]
fn recommended_modes_are_side_safe_for_limit_orders() {
    let (buy_price_mode, buy_qty_mode) = OrderGate::recommended_modes(Side::Buy);
    assert_eq!(buy_price_mode, QuantizeMode::Floor);
    assert_eq!(buy_qty_mode, QuantizeMode::ToZero);

    let (sell_price_mode, sell_qty_mode) = OrderGate::recommended_modes(Side::Sell);
    assert_eq!(sell_price_mode, QuantizeMode::Ceil);
    assert_eq!(sell_qty_mode, QuantizeMode::ToZero);
}

#[test]
fn side_aware_quantization_matches_safety_contract() {
    let gate = OrderGate::default();
    let tick = TickSize(Decimal::from_str_exact("1").unwrap());
    let step = StepSize(Decimal::from_str_exact("0.0001").unwrap());

    let (buy_pm, buy_qm) = OrderGate::recommended_modes(Side::Buy);
    let (buy_price, buy_qty) = gate
        .quantize_limit(
            Side::Buy,
            Decimal::from_str_exact("1000.9").unwrap(),
            Decimal::from_str_exact("0.123456").unwrap(),
            tick,
            step,
            buy_pm,
            buy_qm,
        )
        .unwrap();
    assert_eq!(
        buy_price.as_decimal(),
        Decimal::from_str_exact("1000").unwrap()
    );
    assert_eq!(
        buy_qty.as_decimal(),
        Decimal::from_str_exact("0.1234").unwrap()
    );

    let (sell_pm, sell_qm) = OrderGate::recommended_modes(Side::Sell);
    let (sell_price, sell_qty) = gate
        .quantize_limit(
            Side::Sell,
            Decimal::from_str_exact("1000.1").unwrap(),
            Decimal::from_str_exact("0.123456").unwrap(),
            tick,
            step,
            sell_pm,
            sell_qm,
        )
        .unwrap();
    assert_eq!(
        sell_price.as_decimal(),
        Decimal::from_str_exact("1001").unwrap()
    );
    assert_eq!(
        sell_qty.as_decimal(),
        Decimal::from_str_exact("0.1234").unwrap()
    );
}

#[test]
fn validate_limit_rejects_misaligned_qty() {
    let gate = OrderGate::default();
    let tick = TickSize(Decimal::from_str_exact("1").unwrap());
    let step = StepSize(Decimal::from_str_exact("0.0001").unwrap());

    let err = gate
        .validate_limit(
            Decimal::from_str_exact("1000").unwrap(),
            Decimal::from_str_exact("0.12345").unwrap(),
            tick,
            step,
        )
        .unwrap_err();

    assert!(err.to_string().contains("step") || err.to_string().contains("qty"));
}
