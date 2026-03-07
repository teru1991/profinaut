use std::fs;
use ucel_testkit::ir_issuer_sites::{adapter_map, issuer_site_source_ids, repo_root};

#[test]
fn ir_issuer_sites_contract_matrix() {
    let root = repo_root();
    let inv_ids = issuer_site_source_ids(&root).expect("inventory issuer-site sources");
    let adapters = adapter_map();
    for id in &inv_ids {
        assert!(adapters.contains_key(id), "missing adapter route for {id}");
    }
    let matrix = fs::read_to_string(root.join("ucel/docs/ir/jp_us_issuer_site_matrix.md")).unwrap();
    for id in &inv_ids {
        assert!(matrix.contains(id), "matrix missing source {id}");
    }
}
