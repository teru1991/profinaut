use ucel_testkit::market_data::public_adapter_support_matrix;

#[test]
fn family_split_venues_exist_in_matrix() {
    let matrix = public_adapter_support_matrix();
    for venue in ["binance", "binance-usdm", "binance-coinm", "binance-options"] {
        assert!(matrix.contains_key(venue), "missing {venue}");
    }
}
