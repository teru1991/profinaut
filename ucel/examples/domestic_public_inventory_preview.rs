use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let inventory = ucel_testkit::domestic_public_inventory::load_domestic_public_inventory(&root)
        .expect("load domestic public inventory");
    let summary = ucel_testkit::domestic_public_inventory::summarize_by_venue(&inventory.entries);

    println!("version={}", inventory.version);
    for (venue, counters) in summary {
        let rest = counters.get("api_kind:rest").copied().unwrap_or(0);
        let ws = counters.get("api_kind:ws").copied().unwrap_or(0);
        let core = counters
            .get("surface_class:canonical_core")
            .copied()
            .unwrap_or(0);
        let ext = counters
            .get("surface_class:canonical_extended")
            .copied()
            .unwrap_or(0);
        let vendor = counters
            .get("surface_class:vendor_public_extension")
            .copied()
            .unwrap_or(0);
        println!(
            "venue={venue} rest={rest} ws={ws} canonical_core={core} canonical_extended={ext} vendor_public_extension={vendor}"
        );
    }
}
