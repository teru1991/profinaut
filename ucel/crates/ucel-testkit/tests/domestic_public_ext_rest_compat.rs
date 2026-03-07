use ucel_core::{
    build_vendor_public_rest_typed_envelope, vendor_public_rest_operation_specs,
    VendorPublicRestSchemaVersion,
};

#[test]
fn schema_version_compare_orders_correctly() {
    let v1 = VendorPublicRestSchemaVersion::new(1, 0, 0);
    let v2 = VendorPublicRestSchemaVersion::new(1, 1, 0);
    let v3 = VendorPublicRestSchemaVersion::new(2, 0, 0);
    assert!(v1.compare(v2).is_lt());
    assert!(v2.compare(v3).is_lt());
    assert!(v3.compare(v1).is_gt());
}

#[test]
fn schema_missing_operation_fails_fast() {
    let err = build_vendor_public_rest_typed_envelope(
        "bitbank",
        "bitbank.unknown.rest.op",
        "/x",
        &serde_json::json!({"status":"ok"}),
    )
    .expect_err("unknown operation should fail");
    assert!(err
        .to_string()
        .contains("unknown vendor public rest extension operation"));
}

#[test]
fn known_operations_have_schema_versions() {
    for spec in vendor_public_rest_operation_specs() {
        assert!(
            spec.schema_version.major >= 1,
            "missing major for {}",
            spec.operation_id
        );
    }
}

#[test]
fn additive_payload_still_supported_in_typed_builder() {
    let env = build_vendor_public_rest_typed_envelope(
        "coincheck",
        "coincheck.rest.public.order_books.get",
        "/api/order_books",
        &serde_json::json!({"bids": [["1", "2"]], "asks": [["1.1", "3"]], "new_field": "additive"}),
    )
    .expect("additive field should be tolerated");
    assert_eq!(
        env.metadata.operation_id,
        "coincheck.rest.public.order_books.get"
    );
}
