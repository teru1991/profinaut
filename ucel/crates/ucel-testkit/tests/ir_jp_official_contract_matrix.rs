use std::fs;
use ucel_testkit::ir_jp_official::{adapter_map, jp_official_source_ids, repo_root};

#[test]
fn ir_jp_official_contract_matrix() {
    let root = repo_root();
    let inv_ids = jp_official_source_ids(&root).expect("inventory jp sources");
    let adapters = adapter_map();

    for id in &inv_ids {
        assert!(adapters.contains_key(id), "missing adapter route for {id}");
    }

    let matrix =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_source_matrix.md")).unwrap();
    for id in &inv_ids {
        assert!(matrix.contains(id), "matrix missing source {id}");
    }
}
