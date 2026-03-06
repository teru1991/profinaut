use ucel_testkit::equity::{assert_delayed_or_eod, demo_adapter};
use ucel_equity_core::vendor::EquityVendorAdapter;

#[test]
fn quote_and_bar_normalization_and_latency_class() {
    let adapter = demo_adapter();
    let q = adapter.get_quote("7203.T").unwrap();
    assert!(q.bid > 0.0);
    assert_delayed_or_eod(q.latency);

    let bars = adapter.get_bars("7203.T", "1d", 10).unwrap();
    assert!(!bars.is_empty());
    assert_eq!(bars[0].timeframe, "1d");
    assert_delayed_or_eod(bars[0].latency);
}
