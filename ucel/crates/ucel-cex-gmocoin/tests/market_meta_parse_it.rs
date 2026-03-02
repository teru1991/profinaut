use rust_decimal::Decimal;
use std::str::FromStr;
use std::{fs, path::Path};
use ucel_cex_gmocoin::market_meta::parse_market_meta_snapshot;

fn repo_root() -> std::path::PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("..").join("..").join("..")
}

#[test]
fn parses_fixture_into_valid_market_meta() {
    let fixture = repo_root()
        .join("ucel")
        .join("fixtures")
        .join("market_meta")
        .join("gmocoin")
        .join("public_v1_symbols.json");
    let json = fs::read_to_string(fixture).expect("fixture");

    let snapshot = parse_market_meta_snapshot(&json).expect("parse snapshot");
    assert_eq!(snapshot.markets.len(), 2);

    let btc = snapshot
        .markets
        .iter()
        .find(|m| m.id.raw_symbol == "BTC")
        .expect("BTC row");

    assert_eq!(btc.tick_size, Decimal::from_str("1").unwrap());
    assert_eq!(btc.step_size, Decimal::from_str("0.00001").unwrap());
    assert_eq!(btc.min_qty, Some(Decimal::from_str("0.00001").unwrap()));
    assert_eq!(btc.max_qty, Some(Decimal::from_str("5").unwrap()));
}
