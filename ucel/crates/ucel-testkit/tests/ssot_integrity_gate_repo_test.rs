use std::path::{Path, PathBuf};

use ucel_testkit::run_ssot_integrity_gate;

fn repo_root_from_manifest_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn ssot_integrity_gate_v2_repo_failures_must_be_zero() {
    let repo_root = repo_root_from_manifest_dir();
    let report = run_ssot_integrity_gate(&repo_root).expect("run v2 gate");

    if report.has_failures() {
        panic!("{}", report.format_human_readable());
    }
}
