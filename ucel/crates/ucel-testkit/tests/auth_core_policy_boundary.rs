use ucel_core::{
    AccessSurface, AuthMode, AuthRequestMeta, AuthSurface, ResidencyClass, VenueAccessPolicy,
    VenueAccessScope,
};
use ucel_core::policy::enforce_access;
use ucel_transport::{enforce_auth_boundary, RequestContext};

#[test]
fn policy_blocks_private_before_auth_runtime() {
    let policy = VenueAccessPolicy {
        policy_id: "jp-resident-v1".into(),
        residency: ResidencyClass::JpResident,
        default_scope: VenueAccessScope::PublicOnly,
        entries: vec![],
    };

    let err = enforce_access(&policy, "global_exchange", AccessSurface::PrivateRest).unwrap_err();
    assert_eq!(err.code, ucel_core::ErrorCode::PermissionDenied);
}

#[test]
fn public_request_can_pass_without_secrets() {
    let ctx = RequestContext {
        trace_id: "t".into(),
        request_id: "r".into(),
        run_id: "run".into(),
        op: ucel_core::OpName::FetchTicker,
        venue: "bitbank".into(),
        policy_id: "p".into(),
        key_id: None,
        requires_auth: false,
    };

    assert!(enforce_auth_boundary(&ctx).is_ok());
}

#[test]
fn private_request_without_key_id_fails_fast() {
    let _meta = AuthRequestMeta {
        venue: "bitbank".into(),
        surface: AuthSurface::PrivateRest,
        auth_mode: AuthMode::HmacHeader,
        requires_auth: true,
        request_name: "place_order".into(),
        key_id: None,
    };

    let ctx = RequestContext {
        trace_id: "t".into(),
        request_id: "r".into(),
        run_id: "run".into(),
        op: ucel_core::OpName::PlaceOrder,
        venue: "bitbank".into(),
        policy_id: "p".into(),
        key_id: None,
        requires_auth: true,
    };

    let err = enforce_auth_boundary(&ctx).unwrap_err();
    assert_eq!(err.code, ucel_core::ErrorCode::MissingAuth);
}
