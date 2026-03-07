use ucel_core::{CanonicalOrderBookDelta, CanonicalOrderBookLevel, CanonicalOrderBookSnapshot};
use ucel_testkit::market_data::assert_apply_and_guard;

#[test]
fn snapshot_plus_delta_stays_sorted_and_not_crossed() {
    let snapshot = CanonicalOrderBookSnapshot {
        symbol: "BTCUSDT".into(),
        bids: vec![CanonicalOrderBookLevel {
            price: 100.into(),
            qty: 1.into(),
        }],
        asks: vec![CanonicalOrderBookLevel {
            price: 101.into(),
            qty: 1.into(),
        }],
        sequence: Some(10),
    };
    let delta = CanonicalOrderBookDelta {
        symbol: "BTCUSDT".into(),
        bids: vec![CanonicalOrderBookLevel {
            price: 99.into(),
            qty: 2.into(),
        }],
        asks: vec![CanonicalOrderBookLevel {
            price: 102.into(),
            qty: 2.into(),
        }],
        sequence_start: Some(11),
        sequence_end: Some(11),
    };
    let next = assert_apply_and_guard(&snapshot, &delta);
    assert_eq!(next.sequence, Some(11));
}
