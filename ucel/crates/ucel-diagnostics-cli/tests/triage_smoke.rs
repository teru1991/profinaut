use std::fs;
use std::path::PathBuf;

#[test]
fn cli_sources_exist() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("src/main.rs").exists());
    assert!(root.join("src/args.rs").exists());
    assert!(root.join("src/rbac.rs").exists());
    assert!(root.join("src/audit.rs").exists());
    assert!(root.join("src/export.rs").exists());
}

#[test]
fn runbook_exists() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let runbook = repo_root.join("docs/runbooks/y_support_bundle_runbook.md");
    assert!(runbook.exists());

    let content = fs::read_to_string(runbook).expect("read runbook");
    assert!(content.contains("Break-Glass"));
    assert!(content.contains("audit"));
}
