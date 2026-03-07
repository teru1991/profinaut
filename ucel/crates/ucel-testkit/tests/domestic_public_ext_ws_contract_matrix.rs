use std::collections::{BTreeMap, BTreeSet};

use ucel_registry::hub::{ws::list_vendor_public_ws_extension_operation_ids, ExchangeId};
use ucel_testkit::domestic_public_ws_ext::{load_inventory, repo_root, vendor_ws_entries};

#[test]
fn inventory_vendor_ws_entries_match_registry_listing() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");
    let mut expected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in vendor_ws_entries(&inv) {
        expected
            .entry(e.venue.clone())
            .or_default()
            .insert(e.public_id.clone());
    }

    for (venue, exchange) in [
        ("bitbank", ExchangeId::Bitbank),
        ("bitflyer", ExchangeId::Bitflyer),
        ("coincheck", ExchangeId::Coincheck),
        ("gmocoin", ExchangeId::Gmocoin),
        ("bittrade", ExchangeId::Bittrade),
        ("sbivc", ExchangeId::Sbivc),
    ] {
        let listed = list_vendor_public_ws_extension_operation_ids(exchange)
            .expect("list ids")
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            listed,
            expected.get(venue).cloned().unwrap_or_default(),
            "{venue}"
        );
    }
}

#[test]
fn extension_matrix_counts_match_inventory() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");
    let matrix = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_ws_extension_matrix.md"),
    )
    .expect("matrix");

    let mut expected: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for e in vendor_ws_entries(&inv) {
        let row = expected.entry(e.venue.clone()).or_insert((0, 0, 0));
        row.0 += 1;
        if e.current_repo_status == "implemented" {
            row.1 += 1;
        } else {
            row.2 += 1;
        }
    }

    for line in matrix
        .lines()
        .filter(|l| l.starts_with('|') && !l.contains("---"))
    {
        let cols = line.split('|').map(|x| x.trim()).collect::<Vec<_>>();
        if cols.len() < 6 || cols[1] == "venue" {
            continue;
        }
        let venue = cols[1].to_string();
        let total = cols[2].parse::<usize>().expect("total");
        let implemented = cols[3].parse::<usize>().expect("implemented");
        let not_impl = cols[5].parse::<usize>().expect("not_implemented");
        assert_eq!(
            (total, implemented, not_impl),
            expected.get(&venue).cloned().unwrap_or((0, 0, 0)),
            "{venue}"
        );
    }
}
