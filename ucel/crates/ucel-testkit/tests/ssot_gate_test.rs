use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn ssot_gate_catalog_requires_coverage() {
    // Navigate: crates/ucel-testkit -> crates -> ucel -> repo root
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    ucel_testkit::ssot_gate::run_ssot_gate(&repo_root).unwrap();
}

#[test]
fn ssot_gate_reports_venue_and_id_for_missing_catalog_mapping() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ucel-ssot-gate-{suffix}"));

    fs::create_dir_all(root.join("docs/exchanges/demo"))
        .expect("create docs/exchanges/demo directory");
    fs::create_dir_all(root.join("ucel/coverage")).expect("create ucel/coverage directory");

    fs::write(
        root.join("docs/exchanges/demo/catalog.json"),
        r#"{
  "exchange": "demo",
  "ws_channels": [
    { "id": "openapi.public.ws.ticker.snapshot" }
  ]
}"#,
    )
    .expect("write catalog");

    fs::write(
        root.join("ucel/coverage/demo.yaml"),
        r#"venue: demo
strict: false
entries:
  - id: openapi.public.ws.trade.snapshot
    implemented: false
    tested: false
"#,
    )
    .expect("write coverage");

    let err = ucel_testkit::ssot_gate::run_ssot_gate(&root).expect_err("must fail on missing id");
    assert!(
        err.contains("venue=demo"),
        "error should include venue: {err}"
    );
    assert!(
        err.contains("openapi.public.ws.trade.snapshot"),
        "error should include coverage id: {err}"
    );

    fs::remove_dir_all(&root).expect("cleanup temp tree");
}
