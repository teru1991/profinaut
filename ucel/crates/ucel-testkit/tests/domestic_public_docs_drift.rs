use std::collections::BTreeMap;
use std::fs;

use ucel_testkit::domestic_public_inventory::{
    load_domestic_public_inventory, repo_root, summarize_by_venue,
};

fn parse_matrix_counts(md: &str) -> BTreeMap<String, Vec<usize>> {
    let mut out = BTreeMap::new();
    for line in md
        .lines()
        .filter(|l| l.starts_with("| ") && !l.contains("---"))
    {
        let cols: Vec<_> = line.split('|').map(|c| c.trim()).collect();
        if cols.len() < 9 || cols[1] == "venue" {
            continue;
        }
        let venue = cols[1].to_string();
        let vals = cols[2..9]
            .iter()
            .map(|v| v.parse::<usize>().unwrap_or(0))
            .collect::<Vec<_>>();
        out.insert(venue, vals);
    }
    out
}

#[test]
fn domestic_public_matrix_matches_inventory_json() {
    let root = repo_root();
    let inv = load_domestic_public_inventory(&root).expect("load inventory");
    let summary = summarize_by_venue(&inv.entries);

    let matrix = fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_api_matrix.md"))
        .expect("read matrix");
    let parsed = parse_matrix_counts(&matrix);

    for (venue, counts) in parsed {
        let s = summary.get(&venue).expect("venue in summary");
        let rest = *s.get("api_kind:rest").unwrap_or(&0);
        let ws = *s.get("api_kind:ws").unwrap_or(&0);
        let core = *s.get("surface_class:canonical_core").unwrap_or(&0);
        let ext = *s.get("surface_class:canonical_extended").unwrap_or(&0);
        let vendor = *s.get("surface_class:vendor_public_extension").unwrap_or(&0);
        let not_supported = *s.get("surface_class:not_supported").unwrap_or(&0);
        let not_impl = *s.get("status:not_implemented").unwrap_or(&0);
        assert_eq!(
            counts,
            vec![rest, ws, core, ext, vendor, not_supported, not_impl],
            "venue={venue}"
        );
    }
}

#[test]
fn domestic_public_endpoint_mapping_row_count_matches_inventory() {
    let root = repo_root();
    let inv = load_domestic_public_inventory(&root).expect("load inventory");
    let mapping =
        fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_endpoint_mapping.md"))
            .expect("read mapping");
    let row_count = mapping
        .lines()
        .filter(|l| l.starts_with("| ") && l.contains("`") && !l.contains("public_id"))
        .count();
    assert_eq!(
        row_count,
        inv.entries.len(),
        "mapping row count must match inventory entries"
    );
}
