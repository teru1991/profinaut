use std::path::PathBuf;

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
