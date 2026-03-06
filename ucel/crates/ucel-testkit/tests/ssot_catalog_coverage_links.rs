use std::path::PathBuf;

use ucel_registry::hub::registry;
use ucel_testkit::ssot_consistency::build_consistency_report;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn registry_catalog_keys_are_linked_to_consistency_gate() {
    let report = build_consistency_report(&repo_root()).expect("build report");
    for key in registry::list_registered_catalog_keys() {
        let has_issue = report.issues.iter().any(|i| {
            matches!(
                i.kind,
                ucel_testkit::ssot_consistency::ConsistencyIssueKind::MissingCoverageV2Entry
                    | ucel_testkit::ssot_consistency::ConsistencyIssueKind::CatalogCoverageMismatch
            ) && i.venue == key
        });
        assert!(!has_issue, "catalog/coverage linkage issue for {key}");
    }
}
