use serde_json::json;
use std::path::Path;

use ucel_cex_gmocoin::ws::{GmoCoinPublicWsAdapter, GmoCoinPrivateWsAdapter};
use ucel_cex_gmocoin::rest::GmoCredentials;
use ucel_subscription_planner::{load_coverage_v2, generate_plan_v2};
use ucel_transport::ws::adapter::WsVenueAdapter;
use ucel_ws_rules::load_for_exchange;

fn repo_root() -> std::path::PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("..").join("..").join("..").join("..")
}

#[test]
fn gmocoin_public_coverage_v2_all_families_build_subscribe_ok() {
    let root = repo_root();
    let cov_path = root.join("ucel").join("coverage_v2").join("gmocoin-public.yaml");
    let rules_dir = root.join("ucel").join("crates").join("ucel-ws-rules").join("rules");

    let cov = load_coverage_v2(&cov_path).expect("load coverage_v2");
    let rules = load_for_exchange(&rules_dir, "gmocoin-public");

    // CI-safe: fixed symbol list (GMO public WS expects "BTC" like examples)
    let symbols = vec!["BTC".to_string(), "ETH".to_string()];
    let plan = generate_plan_v2("gmocoin-public", &cov, &symbols, &rules);

    let adapter = GmoCoinPublicWsAdapter::new();

    for k in plan.seed {
        let msgs = adapter.build_subscribe(&k.op_id, k.symbol.as_deref().unwrap_or(""), &k.params);
        assert!(msgs.is_ok(), "build_subscribe failed: key={:?} err={:?}", k, msgs.err());
    }
}

#[test]
fn gmocoin_private_coverage_v2_all_families_build_subscribe_ok() {
    let root = repo_root();
    let cov_path = root.join("ucel").join("coverage_v2").join("gmocoin-private.yaml");
    let rules_dir = root.join("ucel").join("crates").join("ucel-ws-rules").join("rules");

    let cov = load_coverage_v2(&cov_path).expect("load coverage_v2");
    let rules = load_for_exchange(&rules_dir, "gmocoin-private");

    let symbols: Vec<String> = vec![]; // private families are symbol-less
    let plan = generate_plan_v2("gmocoin-private", &cov, &symbols, &rules);

    // creds are not required for build_subscribe test (token is fetched on ws_url/fetch_symbols at runtime)
    let dummy = GmoCredentials { api_key: "DUMMY".into(), api_secret: "DUMMY".into() };
    let adapter = GmoCoinPrivateWsAdapter::new(dummy).expect("adapter");

    for k in plan.seed {
        let msgs = adapter.build_subscribe(&k.op_id, "", &k.params);
        assert!(msgs.is_ok(), "build_subscribe failed: key={:?} err={:?}", k, msgs.err());
    }
}