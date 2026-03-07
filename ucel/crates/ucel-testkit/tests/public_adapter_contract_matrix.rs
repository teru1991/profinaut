use ucel_core::PublicAdapterSupport;
use ucel_testkit::market_data::public_adapter_support_matrix;

#[test]
fn public_adapter_support_state_is_defined_for_target_venues() {
    let matrix = public_adapter_support_matrix();
    assert_eq!(
        matrix.get("binance"),
        Some(&PublicAdapterSupport::Supported)
    );
    assert_eq!(matrix.get("bithumb"), Some(&PublicAdapterSupport::Partial));
    assert_eq!(matrix.get("upbit"), Some(&PublicAdapterSupport::Supported));
    assert!(matrix
        .values()
        .all(|s| *s != PublicAdapterSupport::NotSupported));
}
