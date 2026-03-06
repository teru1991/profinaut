use ucel_registry::hub::errors::HubError;

#[test]
fn blocked_private_venue_maps_to_policy_error_variant() {
    let err = HubError::PrivateWsBlockedByPolicy("sbivc".into());
    assert!(format!("{err}").contains("blocked by policy"));
}

#[test]
fn missing_auth_is_fail_fast_variant() {
    let err = HubError::MissingPrivateWsAuth("orders".into());
    assert!(format!("{err}").contains("missing auth"));
}
