use ucel_core::vendor_public_ws_operation_specs;
use ucel_registry::hub::{ws::list_vendor_public_ws_extension_operation_ids, ExchangeId};
use ucel_testkit::domestic_public_ws_ext::repo_root;

#[test]
fn schema_matrix_contains_all_specs() {
    let root = repo_root();
    let matrix = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_ws_extension_schema_matrix.md"),
    )
    .expect("schema matrix");

    for spec in vendor_public_ws_operation_specs() {
        assert!(
            matrix.contains(spec.operation_id),
            "missing {}",
            spec.operation_id
        );
    }
}

#[test]
fn usage_doc_operations_match_registry() {
    let root = repo_root();
    let usage = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_ws_extension_usage.md"),
    )
    .expect("usage");

    for exchange in [
        ExchangeId::Bitbank,
        ExchangeId::Bitflyer,
        ExchangeId::Coincheck,
        ExchangeId::Gmocoin,
        ExchangeId::Bittrade,
        ExchangeId::Sbivc,
    ] {
        for op in list_vendor_public_ws_extension_operation_ids(exchange).expect("list ids") {
            assert!(usage.contains(&op), "usage missing {}", op);
        }
    }
}
