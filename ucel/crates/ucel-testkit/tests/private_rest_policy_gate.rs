use ucel_core::policy::enforce_access;
use ucel_core::{
    AccessSurface, AuthMode, AuthRequestMeta, AuthSurface, ErrorCode, ResidencyClass,
    VenueAccessPolicy, VenueAccessScope,
};
use ucel_transport::{enforce_auth_boundary, RequestContext};

#[test]
fn blocked_venue_rejects_private_rest_before_network() {
    let policy = VenueAccessPolicy {
        policy_id: "jp-resident-v1".to_string(),
        residency: ResidencyClass::JpResident,
        default_scope: VenueAccessScope::PublicOnly,
        entries: vec![],
    };

    let err = enforce_access(&policy, "sbivc", AccessSurface::PrivateRest).unwrap_err();
    assert_eq!(err.code, ErrorCode::PermissionDenied);
}

#[test]
fn missing_key_id_is_rejected_before_network() {
    let ctx = RequestContext {
        trace_id: "t".to_string(),
        request_id: "r".to_string(),
        run_id: "run".to_string(),
        op: ucel_core::OpName::FetchBalances,
        venue: "bitbank".to_string(),
        policy_id: "p".to_string(),
        key_id: None,
        requires_auth: true,
    };
    let err = enforce_auth_boundary(&ctx).unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);

    let meta = AuthRequestMeta {
        venue: "bitbank".to_string(),
        surface: AuthSurface::PrivateRest,
        auth_mode: AuthMode::HmacHeader,
        requires_auth: true,
        request_name: "get_balances".to_string(),
        key_id: None,
    };
    assert!(meta.validate().is_err());
}

#[test]
fn public_request_is_not_blocked_by_private_gate() {
    let ctx = RequestContext {
        trace_id: "t".to_string(),
        request_id: "r".to_string(),
        run_id: "run".to_string(),
        op: ucel_core::OpName::FetchTicker,
        venue: "bitbank".to_string(),
        policy_id: "p".to_string(),
        key_id: None,
        requires_auth: false,
    };
    assert!(enforce_auth_boundary(&ctx).is_ok());
}
