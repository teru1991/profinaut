use ucel_equity_core::calendar::validate_sessions;
use ucel_equity_core::vendor::EquityVendorAdapter;
use ucel_testkit::equity::demo_adapter;

#[test]
fn calendar_timezone_and_sessions_are_explicit() {
    let adapter = demo_adapter();
    let cal = adapter.get_market_calendar("JP", "2026-03-06").unwrap();
    assert_eq!(cal.timezone, "Asia/Tokyo");
    validate_sessions(&cal).unwrap();
}
