use rust_decimal::Decimal;
use std::str::FromStr;
use std::{fs, path::Path};
use ucel_cex_binance::market_meta::parse_market_meta_snapshot;

fn repo_root() -> std::path::PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("..").join("..").join("..")
}

#[test]
fn parses_fixture_filters_into_valid_market_meta() {
    let fixture = repo_root()
        .join("ucel")
        .join("fixtures")
        .join("market_meta")
        .join("binance")
        .join("spot_exchangeInfo.json");
    let json = fs::read_to_string(fixture).expect("fixture");

    let snapshot = parse_market_meta_snapshot(&json).expect("parse snapshot");
    assert_eq!(snapshot.markets.len(), 1);

    let btcusdt = &snapshot.markets[0];
    assert_eq!(btcusdt.id.raw_symbol, "BTC/USDT");
    assert_eq!(btcusdt.tick_size, Decimal::from_str("0.01").unwrap());
    assert_eq!(btcusdt.step_size, Decimal::from_str("0.00001").unwrap());
    assert_eq!(btcusdt.min_qty, Some(Decimal::from_str("0.00001").unwrap()));
    assert_eq!(btcusdt.max_qty, Some(Decimal::from_str("9000").unwrap()));
    assert_eq!(btcusdt.min_notional, Some(Decimal::from_str("10").unwrap()));
}
