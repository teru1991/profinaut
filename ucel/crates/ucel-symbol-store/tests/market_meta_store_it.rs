use rust_decimal::prelude::FromStr;
use std::time::Duration;
use ucel_symbol_core::{MarketMetaSnapshot, OrderSide};
use ucel_symbol_store::MarketMetaStore;

#[test]
fn apply_snapshot_and_get_and_normalize_order_works() {
    let store = MarketMetaStore::new(Duration::from_secs(60));
    let raw = include_str!("fixtures/market_meta/bitbank_spot.json");
    let snap: MarketMetaSnapshot = serde_json::from_str(raw).expect("fixture must parse");

    let events = store.apply_snapshot_full(snap);
    assert!(!events.is_empty());

    let meta = store
        .get_by_parts(
            ucel_symbol_core::Exchange::Bitbank,
            ucel_symbol_core::MarketType::Spot,
            "BTC/JPY",
        )
        .expect("meta must exist");

    let (p, q) = meta
        .normalize_order(
            rust_decimal::Decimal::from_str("5000.9").unwrap(),
            rust_decimal::Decimal::from_str("0.123456").unwrap(),
            OrderSide::Buy,
        )
        .expect("should satisfy min_notional for this example");

    assert_eq!(p.to_string(), "5000");
    assert_eq!(q.to_string(), "0.1234");
}

#[test]
fn ttl_expiry_removes_entry_on_get() {
    let store = MarketMetaStore::new(Duration::from_millis(1));
    let raw = include_str!("fixtures/market_meta/bitbank_spot.json");
    let snap: MarketMetaSnapshot = serde_json::from_str(raw).expect("fixture must parse");
    store.apply_snapshot_full(snap);

    std::thread::sleep(Duration::from_millis(5));
    let got = store.get_by_parts(
        ucel_symbol_core::Exchange::Bitbank,
        ucel_symbol_core::MarketType::Spot,
        "BTC/JPY",
    );
    assert!(got.is_none());
}
