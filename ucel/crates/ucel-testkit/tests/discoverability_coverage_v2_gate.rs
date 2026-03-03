use std::path::{Path, PathBuf};

use ucel_testkit::coverage_gate::{load_json, public_ws_enabled};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .to_path_buf()
}

#[test]
fn public_ws_true_requires_non_empty_supported_ws_ops_for_selected_exchanges() {
    check_one(
        "deribit",
        ucel_cex_deribit::channels::supported_ws_ops().len(),
    );
    check_one(
        "coincheck",
        ucel_cex_coincheck::channels::supported_ws_ops().len(),
    );
}

fn check_one(exchange: &str, supported_len: usize) {
    let path = repo_root()
        .join("ucel/coverage/coverage_v2/exchanges")
        .join(format!("{exchange}.json"));
    if !path.exists() {
        return;
    }
    let v = load_json(&path).expect("coverage v2 json must parse");
    let ws = public_ws_enabled(&v).expect("public.ws must exist");
    if ws {
        assert!(
            supported_len > 0,
            "{exchange}: public.ws=true but supported_ws_ops is empty"
        );
    }
}
