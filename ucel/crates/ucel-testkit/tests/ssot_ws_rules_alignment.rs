use std::collections::BTreeSet;
use std::path::PathBuf;

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
fn ws_rule_alignment_has_no_policy_or_missing_rule_issues() {
    let report = build_consistency_report(&repo_root()).expect("build report");
    let blocking: BTreeSet<String> = report
        .issues
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                ucel_testkit::ssot_consistency::ConsistencyIssueKind::MissingWsRule
                    | ucel_testkit::ssot_consistency::ConsistencyIssueKind::PolicyEntitlementMismatch
                    | ucel_testkit::ssot_consistency::ConsistencyIssueKind::UnknownCanonicalName
            )
        })
        .map(|i| format!("{:?}:{}", i.kind, i.venue))
        .collect();
    assert!(blocking.is_empty(), "blocking ws issues: {blocking:?}");
}
