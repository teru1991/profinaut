use serde_json::json;
use std::path::Path;

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_testkit::coverage::public_crypto_ws_ops_from_coverage;
use ucel_transport::ws::adapter::{InboundClass, WsVenueAdapter};

fn repo_root() -> std::path::PathBuf {
    // tests are run from crate dir; go up to repo root.
    // ucel/crates/ucel-cex-gmocoin -> ucel workspace root
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("..").join("..")
}

#[test]
fn coverage_ops_are_all_supported_by_build_subscribe() {
    let a = GmoCoinWsAdapter::new();

    let root = repo_root();
    let coverage_dir = root.join("coverage");

    let ops = public_crypto_ws_ops_from_coverage(&coverage_dir, "gmocoin").unwrap();
    assert!(!ops.is_empty(), "gmocoin coverage has no public ws ops");

    for op in ops {
        // canonical symbol例（GMOはBTC/JPYが必ずあるとは限らないが、build_subscribe契約検証のため固定）
        // 実運用は symbols.rs で得た symbol を使う。
        let res = a.build_subscribe(&op, "BTC/JPY", &json!({}));
        assert!(
            res.is_ok(),
            "op_id from coverage not supported by build_subscribe: op_id={op}, err={:?}",
            res.err()
        );
    }
}

#[test]
fn classify_inbound_maps_channel_to_op_id_and_symbol() {
    let a = GmoCoinWsAdapter::new();

    let raw = json!({
        "channel": "ticker",
        "symbol": "BTC_JPY"
    })
    .to_string();

    match a.classify_inbound(raw.as_bytes()) {
        InboundClass::Data {
            op_id,
            symbol,
            params_canon_hint,
        } => {
            assert_eq!(op_id.unwrap(), "crypto.public.ws.ticker.update");
            assert_eq!(symbol.unwrap(), "BTC/JPY");
            assert_eq!(params_canon_hint.unwrap(), "{}");
        }
        other => panic!("unexpected inbound class: {other:?}"),
    }
}
