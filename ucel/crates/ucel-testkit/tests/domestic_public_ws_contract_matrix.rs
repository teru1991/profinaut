use std::collections::{BTreeMap, BTreeSet};

use ucel_registry::hub::{ws::list_pending_vendor_public_ws_extension_channel_ids, ExchangeId};
use ucel_testkit::domestic_public_ws::{load_inventory, repo_root, ws_entries};

#[test]
fn ws_inventory_surface_counts_match_matrix_doc() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");
    let matrix =
        std::fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_ws_matrix.md"))
            .expect("read matrix");

    let mut expected: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    for e in ws_entries(&inv) {
        let row = expected.entry(e.venue.clone()).or_insert((0, 0, 0, 0));
        row.0 += 1;
        if e.surface_class == "canonical_core" {
            row.1 += 1;
        }
        if e.surface_class == "vendor_public_extension" {
            row.2 += 1;
        }
        if e.current_repo_status == "implemented" {
            row.3 += 1;
        }
    }

    for line in matrix
        .lines()
        .filter(|l| l.starts_with('|') && !l.contains("---"))
    {
        let cols = line.split('|').map(|x| x.trim()).collect::<Vec<_>>();
        if cols.len() < 7 || cols[1] == "venue" {
            continue;
        }
        let venue = cols[1].to_string();
        let total = cols[2].parse::<usize>().expect("total");
        let core = cols[3].parse::<usize>().expect("canonical_core");
        let vendor = cols[4].parse::<usize>().expect("vendor_extension");
        let implemented = cols[5].parse::<usize>().expect("implemented");
        let expected_row = expected.get(&venue).cloned().unwrap_or((0, 0, 0, 0));
        assert_eq!(
            (total, core, vendor, implemented),
            expected_row,
            "venue={venue}"
        );
    }
}

#[test]
fn pending_vendor_extension_channels_are_visible_via_registry() {
    let root = repo_root();
    let inv = load_inventory(&root).expect("load inventory");

    let mut inventory: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in ws_entries(&inv)
        .into_iter()
        .filter(|e| e.surface_class == "vendor_public_extension")
    {
        inventory
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
        let listed = list_pending_vendor_public_ws_extension_channel_ids(exchange)
            .expect("list pending vendor extension channels")
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            listed,
            inventory.get(venue).cloned().unwrap_or_default(),
            "venue={venue}"
        );
    }
}
