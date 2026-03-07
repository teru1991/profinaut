use std::collections::BTreeSet;

use ucel_core::vendor_public_rest_operation_specs;
use ucel_testkit::domestic_public_rest_ext::repo_root;

#[test]
fn schema_matrix_matches_operation_specs() {
    let root = repo_root();
    let matrix = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_rest_extension_schema_matrix.md"),
    )
    .expect("read schema matrix");

    let mut rows = BTreeSet::new();
    for line in matrix
        .lines()
        .filter(|x| x.starts_with('|') && !x.contains("---"))
    {
        let cols = line.split('|').map(|x| x.trim()).collect::<Vec<_>>();
        if cols.len() < 8 || cols[1] == "venue" {
            continue;
        }
        rows.insert(format!("{}|{}", cols[1], cols[2].trim_matches('`')));
    }

    for spec in vendor_public_rest_operation_specs() {
        let key = format!("{}|{}", spec.venue, spec.operation_id);
        assert!(rows.contains(&key), "missing schema matrix row: {key}");
    }
}

#[test]
fn usage_doc_operations_exist_in_specs() {
    let root = repo_root();
    let usage = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_rest_extension_usage.md"),
    )
    .expect("read usage doc");

    for spec in vendor_public_rest_operation_specs() {
        assert!(
            usage.contains(spec.operation_id),
            "usage doc missing operation {}",
            spec.operation_id
        );
    }
}
