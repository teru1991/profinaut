use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn v1_coverage_is_not_used_for_ci_gating() {
    let policy = std::fs::read_to_string(repo_root().join("ucel/docs/policies/coverage_policy.md"))
        .expect("coverage_policy.md must exist");
    assert!(
        policy.contains("coverage_v2"),
        "policy must reference coverage_v2"
    );
    assert!(
        policy.to_lowercase().contains("legacy"),
        "policy must mark v1 as legacy"
    );

    let strict_path = repo_root().join("ucel/coverage/coverage_v2/strict_venues.json");
    let strict_raw = std::fs::read_to_string(strict_path).expect("strict_venues.json must exist");
    let strict: serde_json::Value = serde_json::from_str(&strict_raw).expect("strict json");
    let strict_ws = strict
        .get("strict_ws_golden")
        .and_then(|v| v.as_array())
        .expect("strict_ws_golden must be array");
    assert!(
        !strict_ws
            .iter()
            .filter_map(|v| v.as_str())
            .any(|v| v == "sbivc"),
        "sbivc must not be strict"
    );
}
