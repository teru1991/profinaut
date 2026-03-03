use std::path::{Path, PathBuf};

use ucel_testkit::coverage_gate::{
    assert_domestic_requirements, assert_overseas_requirements, load_json, ws_ops,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .to_path_buf()
}

fn coverage_v2_path(exchange: &str) -> PathBuf {
    repo_root()
        .join("ucel/coverage/coverage_v2/exchanges")
        .join(format!("{exchange}.json"))
}

fn channels_path(exchange: &str) -> PathBuf {
    repo_root()
        .join("ucel/crates")
        .join(format!("ucel-cex-{exchange}"))
        .join("src/channels/mod.rs")
}

fn parse_supported_ws_ops(channels_mod: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_str = false;
    let mut cur = String::new();
    for ch in channels_mod.chars() {
        if ch == '"' {
            if in_str {
                out.push(cur.clone());
                cur.clear();
                in_str = false;
            } else {
                in_str = true;
            }
            continue;
        }
        if in_str {
            cur.push(ch);
        }
    }
    out
}

#[test]
fn coverage_v2_domestic_must_have_private_except_sbivc() {
    for ex in ["gmocoin", "bitbank", "bitflyer", "coincheck"] {
        let path = coverage_v2_path(ex);
        let v = load_json(&path).expect("coverage v2 must parse");
        assert_domestic_requirements(ex, &v).expect("domestic requirements");
    }
}

#[test]
fn coverage_v2_overseas_public_only_is_ok() {
    for ex in ["bybit", "okx", "coinbase", "kraken"] {
        let path = coverage_v2_path(ex);
        if !path.exists() {
            continue;
        }
        let v = load_json(&path).expect("coverage v2 must parse");
        assert_overseas_requirements(ex, &v).expect("overseas requirements");
    }
}

#[test]
fn supported_ws_ops_aligns_with_coverage_v2() {
    for ex in [
        "coincheck",
        "coinbase",
        "bybit",
        "bitget",
        "bitmex",
        "bittrade",
        "deribit",
        "okx",
        "htx",
        "kraken",
        "upbit",
        "sbivc",
        "binance-usdm",
        "binance-coinm",
        "binance-options",
    ] {
        let cpath = coverage_v2_path(ex);
        if !cpath.exists() {
            continue;
        }
        let channels = channels_path(ex);
        if !channels.exists() {
            continue;
        }
        let v = load_json(&cpath).expect("coverage v2 parse");
        let expected = ws_ops(&v);
        let actual_raw = std::fs::read_to_string(channels).expect("channels read");
        let actual = parse_supported_ws_ops(&actual_raw);

        if expected.is_empty() {
            assert!(actual.is_empty(), "{ex} must declare empty ws ops");
        } else {
            for op in expected {
                assert!(
                    actual.contains(&op),
                    "{ex} missing ws op {op}; actual={actual:?}"
                );
            }
        }
    }
}
