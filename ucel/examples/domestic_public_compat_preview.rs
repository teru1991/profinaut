use std::path::Path;
use ucel_testkit::domestic_public_compat::{load_inventory_and_lock, summarize_inventory};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let (inv, lock) = load_inventory_and_lock(&root).expect("load inventory+lock");
    let (summary, by_venue, _) = summarize_inventory(&inv);

    println!("domestic_public compatibility preview");
    println!("version={} lock_version={}", inv.version, lock.version);
    println!(
        "total={} rest={} ws={} core={} ext={} vendor={} not_supported={}",
        summary.total_entries,
        summary.rest_entries,
        summary.ws_entries,
        summary.canonical_core,
        summary.canonical_extended,
        summary.vendor_public_extension,
        summary.not_supported
    );
    for (venue, c) in by_venue {
        println!(
            "{venue}: total={} rest={} ws={} core={} ext={} vendor={}",
            c.total_entries,
            c.rest_entries,
            c.ws_entries,
            c.canonical_core,
            c.canonical_extended,
            c.vendor_public_extension
        );
    }
}
