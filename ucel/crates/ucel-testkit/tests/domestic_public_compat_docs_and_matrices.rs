use std::fs;
use ucel_testkit::domestic_public_compat::{collect_docs_matrix_counts, default_repo_root, load_inventory_and_lock, summarize_inventory};

#[test]
fn domestic_public_compat_docs_summary_matches_inventory() {
    let root = default_repo_root();
    let (inv, _) = load_inventory_and_lock(&root).expect("load inventory");
    let (s, _, _) = summarize_inventory(&inv);
    let docs = collect_docs_matrix_counts(&root).expect("parse compat matrix");

    assert_eq!(docs.get("summary.total_entries").copied(), Some(s.total_entries));
    assert_eq!(docs.get("summary.rest_entries").copied(), Some(s.rest_entries));
    assert_eq!(docs.get("summary.ws_entries").copied(), Some(s.ws_entries));
    assert_eq!(docs.get("summary.canonical_core").copied(), Some(s.canonical_core));
    assert_eq!(docs.get("summary.canonical_extended").copied(), Some(s.canonical_extended));
    assert_eq!(docs.get("summary.vendor_public_extension").copied(), Some(s.vendor_public_extension));
    assert_eq!(docs.get("summary.not_supported").copied(), Some(s.not_supported));

    let report = fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_final_support_report.md")).expect("read report");
    assert!(report.contains("partial/not_implemented: 0"));
}
