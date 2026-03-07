use ucel_testkit::domestic_public_compat::{collect_schema_runtime_versions, default_repo_root};

#[test]
fn domestic_public_compat_schema_runtime_documents_present() {
    let root = default_repo_root();
    let versions = collect_schema_runtime_versions(&root).expect("collect");
    assert_eq!(versions.len(), 5, "schema/runtime policy docs are incomplete");
}
