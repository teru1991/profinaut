use std::path::{Path, PathBuf};

use ucel_testkit::coverage_gate::load_json;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .to_path_buf()
}

fn p(rel: &str) -> PathBuf {
    repo_root().join(rel)
}

#[test]
fn sbivc_is_the_only_domestic_public_only_exception() {
    let sb = p("ucel/coverage/coverage_v2/exchanges/sbivc.json");
    let sbv = load_json(&sb).expect("sbivc coverage must parse");
    let sbivc_private_enabled = sbv
        .get("private")
        .and_then(|x| x.get("enabled"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    assert!(
        !sbivc_private_enabled,
        "sbivc must remain temporary public_only exception"
    );

    for ex in ["gmocoin", "bitbank", "bitflyer", "coincheck"] {
        let path = p(&format!("ucel/coverage/coverage_v2/exchanges/{ex}.json"));
        let v = load_json(&path).expect("coverage must parse");
        let priv_enabled = v
            .get("private")
            .and_then(|x| x.get("enabled"))
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        assert!(priv_enabled, "{ex} must not be public_only");
    }
}
