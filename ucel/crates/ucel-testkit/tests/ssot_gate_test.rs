use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn ssot_gate_catalog_requires_v2_coverage() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    ucel_testkit::ssot_gate::run_ssot_gate(&repo_root).unwrap();
}

#[test]
fn ssot_gate_reports_missing_v2_coverage_file() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ucel-ssot-gate-{suffix}"));

    fs::create_dir_all(root.join("docs/exchanges/demo")).expect("create docs/exchanges/demo");
    fs::create_dir_all(root.join("ucel/coverage/coverage_v2/exchanges")).expect("create exchanges");

    fs::write(
        root.join("docs/exchanges/demo/catalog.json"),
        r#"{
  "exchange": "demo",
  "ws_channels": [
    { "id": "crypto.public.ws.ticker.update" }
  ]
}"#,
    )
    .expect("write catalog");

    fs::write(
        root.join("ucel/coverage/coverage_v2/strict_venues.json"),
        r#"{"strict_ws_golden":["demo"],"strict_symbol_master":[]}"#,
    )
    .expect("write strict_venues.json");

    let err = ucel_testkit::ssot_gate::run_ssot_gate(&root).expect_err("must fail");
    assert!(
        err.contains("venue=demo"),
        "error should include venue: {err}"
    );
    assert!(
        err.contains("coverage_v2/exchanges/demo.json"),
        "error should include missing v2 coverage path: {err}"
    );

    fs::remove_dir_all(&root).expect("cleanup temp tree");
}
