use ucel_core::{validate_trade, CanonicalTrade, Side};

#[test]
fn malformed_numeric_like_zero_qty_is_error_not_panic() {
    let trade = CanonicalTrade {
        symbol: "BTCUSDT".into(),
        trade_id: "1".into(),
        price: 100.into(),
        qty: 0.into(),
        side: Side::Buy,
        ts_event: Some(1),
    };
    let err = validate_trade(&trade).unwrap_err();
    assert!(err.message.contains("qty"));
}
