use std::collections::BTreeSet;

use ucel_testkit::domestic_public_inventory::{load_domestic_public_inventory, repo_root};

#[test]
fn domestic_public_surface_classification_is_single_and_valid() {
    let root = repo_root();
    let inv = load_domestic_public_inventory(&root).expect("load inventory");

    let mut seen = BTreeSet::new();
    for e in &inv.entries {
        let key = format!("{}|{}|{}", e.venue, e.api_kind, e.public_id);
        assert!(seen.insert(key), "duplicate venue+api_kind+public_id");
        assert!(
            matches!(
                e.surface_class.as_str(),
                "canonical_core"
                    | "canonical_extended"
                    | "vendor_public_extension"
                    | "not_supported"
            ),
            "invalid surface_class={} for {}",
            e.surface_class,
            e.public_id
        );
    }
}

#[test]
fn vendor_public_call_not_overused_for_core_categories() {
    let root = repo_root();
    let inv = load_domestic_public_inventory(&root).expect("load inventory");
    for e in &inv.entries {
        let core_cat = matches!(
            e.category.as_str(),
            "ticker" | "trades" | "orderbook" | "candles"
        );
        assert!(
            !(core_cat && e.canonical_surface == "vendor_public_call"),
            "core category wrongly assigned vendor_public_call: {}",
            e.public_id
        );
    }
}
