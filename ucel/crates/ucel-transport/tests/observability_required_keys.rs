use ucel_transport::obs::{span_required, ObsRequiredKeys};

#[test]
fn required_keys_reject_empty() {
    let err = ObsRequiredKeys::try_new("binance", "c1", "book", "", "run1").unwrap_err();
    assert!(err.message.contains("required keys"));
}

#[test]
fn required_keys_span_constructs() {
    let k = ObsRequiredKeys::try_new("binance", "c1", "book", "BTCUSDT", "run1").unwrap();
    let s = span_required("test_span", &k);
    let _entered = s.enter();
}
