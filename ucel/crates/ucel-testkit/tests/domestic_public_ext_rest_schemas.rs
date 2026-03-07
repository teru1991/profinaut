use ucel_testkit::domestic_public_rest_ext::{
    assert_payload_shape, assert_schema_present, build_fixture_envelopes, repo_root,
};

#[test]
fn extension_fixture_envelopes_are_typed_and_complete() {
    let root = repo_root();
    let envelopes = build_fixture_envelopes(&root).expect("build envelopes from fixtures");
    assert!(!envelopes.is_empty(), "fixtures must not be empty");

    for env in envelopes {
        assert!(assert_schema_present(env.schema_version));
        assert!(!env.metadata.venue.is_empty());
        assert!(!env.metadata.operation_id.is_empty());
        assert!(!env.metadata.source_endpoint.is_empty());
        assert!(!env.metadata.inventory_public_id.is_empty());

        let payload_json = serde_json::to_value(&env.typed_payload).expect("serialize payload");
        assert!(assert_payload_shape(env.payload_type, &payload_json));

        let payload_str = payload_json.to_string();
        assert!(
            !payload_str.contains("raw_payload") && !payload_str.contains("passthrough"),
            "raw passthrough marker detected for {}",
            env.metadata.operation_id
        );
    }
}
