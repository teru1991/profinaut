use ucel_equity_adapter_demo::DemoEquityAdapter;
use ucel_equity_core::vendor::EquityVendorAdapter;

pub fn demo_adapter() -> DemoEquityAdapter {
    DemoEquityAdapter::default()
}

pub fn assert_delayed_or_eod(latency: ucel_core::EquityLatencyClass) {
    assert!(matches!(latency, ucel_core::EquityLatencyClass::Delayed | ucel_core::EquityLatencyClass::EndOfDay));
}

pub fn assert_vendor_supported_all(adapter: &dyn EquityVendorAdapter) {
    let cap = adapter.capabilities();
    assert!(matches!(cap.support.quotes, ucel_core::EquitySupport::Supported));
    assert!(matches!(cap.support.bars_intraday, ucel_core::EquitySupport::Supported));
    assert!(matches!(cap.support.calendar, ucel_core::EquitySupport::Supported));
}
