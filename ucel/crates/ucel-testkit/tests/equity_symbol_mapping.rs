use ucel_equity_core::vendor::EquityVendorAdapter;
use ucel_testkit::equity::demo_adapter;

#[test]
fn symbol_mapping_and_ambiguity_handling() {
    let adapter = demo_adapter();
    let sym = adapter.resolve_symbol("7203.T").unwrap();
    assert_eq!(sym.canonical, "7203");

    let missing = adapter.resolve_symbol("UNKNOWN");
    assert!(missing.is_err());
}
