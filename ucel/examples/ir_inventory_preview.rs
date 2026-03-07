use std::path::Path;
use ucel_testkit::ir_inventory::{inventory_counts, load_ir_inventory};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let inv = load_ir_inventory(&root).expect("load inventory");
    let counts = inventory_counts(&inv);

    println!("ir_inventory version={}", inv.version);
    println!("markets={:?}", inv.markets);
    println!(
        "sources={} identities={} documents={}",
        counts["sources"], counts["identities"], counts["documents"]
    );
    println!(
        "implemented={} partial={} not_implemented={}",
        counts["implemented"], counts["partial"], counts["not_implemented"]
    );
}
