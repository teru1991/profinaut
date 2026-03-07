use std::collections::{BTreeMap, BTreeSet};

use ucel_registry::hub::{rest::list_vendor_public_rest_extension_operation_ids, ExchangeId};
use ucel_testkit::domestic_public_rest_ext::{
    load_inventory, operation_spec_map, repo_root, vendor_rest_inventory_entries,
};

#[test]
fn inventory_vendor_rest_entries_are_all_implemented_as_specs() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");
    let specs = operation_spec_map();

    for entry in vendor_rest_inventory_entries(&inv) {
        let key = format!("{}|{}", entry.venue, entry.public_id);
        assert!(specs.contains_key(&key), "missing spec: {key}");
    }
}

#[test]
fn registry_reachable_operation_set_matches_inventory() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");

    let mut inventory_by_venue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in vendor_rest_inventory_entries(&inv) {
        inventory_by_venue
            .entry(entry.venue.clone())
            .or_default()
            .insert(entry.public_id.clone());
    }

    let exchange_map = [
        ("bitbank", ExchangeId::Bitbank),
        ("bitflyer", ExchangeId::Bitflyer),
        ("coincheck", ExchangeId::Coincheck),
        ("gmocoin", ExchangeId::Gmocoin),
        ("bittrade", ExchangeId::Bittrade),
        ("sbivc", ExchangeId::Sbivc),
    ];

    for (venue, exchange) in exchange_map {
        let listed = list_vendor_public_rest_extension_operation_ids(exchange)
            .expect("list vendor public extension ids")
            .into_iter()
            .collect::<BTreeSet<_>>();
        let expected = inventory_by_venue.get(venue).cloned().unwrap_or_default();
        assert_eq!(listed, expected, "venue={venue}");
    }
}

#[test]
fn docs_matrix_counts_match_inventory_status_counts() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");
    let matrix = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_rest_extension_matrix.md"),
    )
    .expect("read extension matrix");

    let mut expected: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for e in vendor_rest_inventory_entries(&inv) {
        let counters = expected.entry(e.venue.clone()).or_insert((0, 0, 0));
        counters.0 += 1;
        if e.current_repo_status == "implemented" {
            counters.1 += 1;
        } else {
            counters.2 += 1;
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
        let not_implemented = cols[5].parse::<usize>().expect("not_implemented");
        let exp = expected.get(&venue).cloned().unwrap_or((0, 0, 0));
        assert_eq!((total, implemented, not_implemented), (exp.0, exp.1, exp.2));
    }
}
