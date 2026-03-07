use std::fs;
use ucel_testkit::ir_inventory::{inventory_counts, load_ir_inventory, repo_root};

fn parse_num(md: &str, key: &str) -> usize {
    md.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}:")))
        .map(|x| x.trim().parse::<usize>().unwrap())
        .unwrap()
}

#[test]
fn ir_docs_drift() {
    let root = repo_root();
    let inv = load_ir_inventory(&root).expect("load inventory");
    let counts = inventory_counts(&inv);

    let matrix = fs::read_to_string(root.join("ucel/docs/ir/jp_us_ir_source_matrix.md")).unwrap();
    assert_eq!(parse_num(&matrix, "summary.sources.total"), counts["sources"]);
    assert_eq!(parse_num(&matrix, "summary.identities.total"), counts["identities"]);
    assert_eq!(parse_num(&matrix, "summary.documents.total"), counts["documents"]);

    let docs = fs::read_to_string(root.join("ucel/docs/ir/ir_document_taxonomy.md")).unwrap();
    assert_eq!(
        parse_num(&docs, "summary.document_rows.total"),
        inv.documents.len()
    );

    let ids = fs::read_to_string(root.join("ucel/docs/ir/ir_identity_mapping_matrix.md")).unwrap();
    assert_eq!(
        parse_num(&ids, "summary.identity_rows.total"),
        inv.identities.len()
    );

    let policy = fs::read_to_string(root.join("ucel/docs/ir/ir_access_policy.md")).unwrap();
    for class in [
        "free_public_noauth_allowed",
        "free_public_noauth_review_required",
        "excluded_paid_or_contract",
        "excluded_login_required",
        "excluded_policy_blocked",
    ] {
        assert!(policy.contains(class), "missing class in doc {class}");
    }
}
