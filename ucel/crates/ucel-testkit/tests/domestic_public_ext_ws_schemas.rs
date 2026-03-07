use ucel_core::{
    build_vendor_public_ws_typed_envelope, vendor_public_ws_operation_specs,
    VendorPublicWsPayloadType,
};
use ucel_testkit::domestic_public_ws_ext::{load_fixtures, repo_root};

#[test]
fn all_fixture_cases_build_typed_envelopes_with_schema_and_metadata() {
    let root = repo_root();
    let fixtures = load_fixtures(&root).expect("fixtures");
    for case in fixtures.cases {
        let env = build_vendor_public_ws_typed_envelope(
            &case.venue,
            &case.operation_id,
            &case.source_channel,
            case.payload,
        )
        .expect("typed envelope");
        assert!(env.schema_version.major > 0);
        assert!(!env.metadata.venue.is_empty());
        assert!(!env.metadata.operation_id.is_empty());
        assert!(!env.metadata.source_channel.is_empty());
        assert!(!env.metadata.inventory_public_id.is_empty());
    }
}

#[test]
fn payload_type_matches_expected_shape() {
    for spec in vendor_public_ws_operation_specs() {
        let payload = match spec.payload_type {
            VendorPublicWsPayloadType::Object
            | VendorPublicWsPayloadType::EnumLikeObject
            | VendorPublicWsPayloadType::SnapshotAndDelta => serde_json::json!({"k":"v"}),
            VendorPublicWsPayloadType::Array | VendorPublicWsPayloadType::EventSeries => {
                serde_json::json!([{"k":"v"}])
            }
        };
        build_vendor_public_ws_typed_envelope(
            spec.venue,
            spec.operation_id,
            spec.source_channel,
            payload,
        )
        .expect("shape compatible");
    }
}
