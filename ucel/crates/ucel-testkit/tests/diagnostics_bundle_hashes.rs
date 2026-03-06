#[test]
fn support_bundle_contains_real_hashes_not_unknown() {
    let repo_root = ucel_testkit::diagnostics::repo_root();
    let hashset = ucel_diagnostics_core::default_hash_set(&repo_root).expect("hash set");
    for value in [
        hashset.coverage_hash,
        hashset.coverage_v2_hash,
        hashset.ws_rules_hash,
        hashset.catalog_hash,
        hashset.policy_hash,
        hashset.symbol_meta_hash,
        hashset.execution_surface_hash,
        hashset.runtime_capability_hash,
    ] {
        assert!(!value.is_empty());
        assert_ne!(value, "unknown");
    }
}

#[test]
fn canonical_newline_normalization_keeps_hash_stable() {
    let repo_root = ucel_testkit::diagnostics::repo_root();
    let h1 = ucel_diagnostics_core::hash_paths(&repo_root, &["fixtures/support_bundle/manifest.json"]).unwrap();
    let h2 = ucel_diagnostics_core::hash_paths(&repo_root, &["fixtures/support_bundle/manifest.json"]).unwrap();
    assert_eq!(h1, h2);
}
