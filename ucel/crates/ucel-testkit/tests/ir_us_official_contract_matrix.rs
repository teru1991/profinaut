use ucel_testkit::ir_us_official::{adapter_map, repo_root, us_official_source_ids};

#[test]
fn ir_us_official_contract_matrix() {
    let root = repo_root();
    let inventory_ids = us_official_source_ids(&root).expect("load us ids");
    let adapters = adapter_map();

    for id in &inventory_ids {
        assert!(adapters.contains_key(id), "missing adapter for {id}");
    }

    let doc = std::fs::read_to_string(root.join("ucel/docs/ir/us_official_source_matrix.md"))
        .expect("read matrix");
    assert!(doc.contains("summary.sources: 1"));
    assert!(doc.contains("summary.implemented: 1"));
    assert!(doc.contains("sec_edgar_submissions_api"));
}
