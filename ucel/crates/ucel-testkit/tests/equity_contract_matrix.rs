use ucel_testkit::equity::{assert_vendor_supported_all, demo_adapter};
use ucel_equity_core::vendor::EquityVendorAdapter;

#[test]
fn vendor_matrix_has_supported_demo_vendor() {
    let adapter = demo_adapter();
    assert_vendor_supported_all(&adapter);
    let cap = adapter.capabilities();
    assert_eq!(cap.vendor_id, "demo-equity");
}
