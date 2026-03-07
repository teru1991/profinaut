use std::collections::BTreeSet;
use ucel_testkit::ir_inventory::{load_ir_inventory, repo_root};

#[test]
fn ir_taxonomy_gate() {
    let root = repo_root();
    let inv = load_ir_inventory(&root).expect("load inventory");

    for s in &inv.sources {
        assert!(!s.source_kind.is_empty());
        assert!(!s.source_family.is_empty());
        assert!(!s.issuer_identity_kind.is_empty());
        assert!(!s.document_family.is_empty());
        assert!(!s.artifact_kind.is_empty());
    }

    let mut seen = BTreeSet::new();
    for d in &inv.documents {
        let key = format!(
            "{}|{}|{}|{}",
            d.market, d.source_id, d.document_family, d.artifact_kind
        );
        assert!(seen.insert(key), "duplicate document taxonomy row");
    }
}
