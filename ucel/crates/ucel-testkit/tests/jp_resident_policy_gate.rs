use std::collections::BTreeSet;
use std::path::PathBuf;

use ucel_testkit::coverage_v2::{
    infer_exchange_id, jp_scope_for_venue, list_exchange_jsons, load_jp_resident_access, load_json,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn jp_resident_policy_json_exists_and_parses() {
    let root = repo_root();
    let policy = load_jp_resident_access(&root).expect("policy json parse");
    assert_eq!(policy.policy_id, "jp-resident-v1");
    assert_eq!(policy.residency, "jp_resident");
    assert_eq!(policy.default_scope, "public_only");
}

#[test]
fn coverage_policy_and_json_are_aligned_for_domestic_rules() {
    let root = repo_root();
    let policy = load_jp_resident_access(&root).expect("policy json parse");
    let md = std::fs::read_to_string(root.join("ucel/docs/policies/coverage_policy.md"))
        .expect("coverage policy");

    for venue in ["bitbank", "bitflyer", "coincheck", "gmocoin"] {
        assert!(
            md.contains(venue),
            "coverage_policy.md must mention {venue}"
        );
        assert_eq!(jp_scope_for_venue(&policy, venue), "public_private");
    }
    assert!(
        md.contains("sbivc"),
        "coverage_policy.md must mention sbivc exception"
    );
    assert_eq!(jp_scope_for_venue(&policy, "sbivc"), "public_only");
}

#[test]
fn unspecified_venues_default_to_public_only() {
    let root = repo_root();
    let policy = load_jp_resident_access(&root).expect("policy json parse");
    let explicit: BTreeSet<String> = policy
        .entries
        .iter()
        .map(|entry| entry.venue.to_ascii_lowercase())
        .collect();

    let cov_root = root.join("ucel/coverage/coverage_v2");
    for path in list_exchange_jsons(&cov_root).expect("coverage files") {
        let json = load_json(&path).expect("valid exchange coverage json");
        let venue = infer_exchange_id(&path, &json).to_ascii_lowercase();
        if !explicit.contains(&venue) {
            assert_eq!(
                jp_scope_for_venue(&policy, &venue),
                "public_only",
                "venue={venue}"
            );
        }
    }
}

#[test]
fn malformed_shape_fails_to_parse() {
    let bad = r#"{"policy_id":"x","residency":"jp_resident","default_scope":"public_only","entries":"oops"}"#;
    let parsed: Result<ucel_testkit::coverage_v2::JpResidentAccessPolicy, _> =
        serde_json::from_str(bad);
    assert!(parsed.is_err());
}
