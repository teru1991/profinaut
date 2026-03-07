use std::path::PathBuf;

use ucel_testkit::ssot_consistency::{assert_consistent, build_consistency_report};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn ssot_consistency_report_builds_and_has_no_blocking_issues() {
    let report = build_consistency_report(&repo_root()).expect("build report");
    for w in &report.warnings {
        assert!(
            matches!(
                w.kind,
                ucel_testkit::ssot_consistency::ConsistencyIssueKind::MissingCoverageV2Entry
                    | ucel_testkit::ssot_consistency::ConsistencyIssueKind::MissingWsRule
            ),
            "unexpected warning kind: {:?}",
            w.kind
        );
    }
    assert_consistent(&report).expect("no ssot consistency issues");
}
