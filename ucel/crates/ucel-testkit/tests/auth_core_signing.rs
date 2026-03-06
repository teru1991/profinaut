use std::collections::HashMap;

use ucel_core::{AuthMaterial, AuthMode, AuthRequestMeta, AuthSurface, ErrorCode, SecretRef};
use ucel_testkit::auth::{
    EchoSigner, MonotonicNonceProvider, SequenceIdempotencyProvider, StaticSecretResolver,
};
use ucel_transport::auth::AuthRuntime;

#[test]
fn resolve_and_sign_builds_context_for_hmac_header() {
    let mut material_by_key = HashMap::new();
    material_by_key.insert(
        "k1".into(),
        AuthMaterial {
            api_key: Some("api-key".into()),
            api_secret: Some("api-secret".into()),
            ..Default::default()
        },
    );
    let resolver = StaticSecretResolver { material_by_key };
    let runtime = AuthRuntime::default();
    let parts = runtime
        .resolve_and_sign(
            &resolver,
            &MonotonicNonceProvider::default(),
            &SequenceIdempotencyProvider::default(),
            &EchoSigner,
            &SecretRef {
                key_id: Some("k1".into()),
                alias: None,
            },
            &AuthRequestMeta {
                venue: "bitbank".into(),
                surface: AuthSurface::PrivateRest,
                auth_mode: AuthMode::HmacHeader,
                requires_auth: true,
                request_name: "place_order".into(),
                key_id: Some("k1".into()),
            },
            "POST",
            "/private/order",
            "symbol=BTC_JPY",
            "{\"qty\":1}",
            1700000000,
        )
        .unwrap();

    assert!(parts.headers.contains_key("x-auth-preview"));
    assert_eq!(parts.headers.get("x-nonce"), Some(&"1".to_string()));
    assert!(parts.headers.contains_key("x-idempotency-key"));
}

#[test]
fn missing_key_id_fails_fast() {
    let runtime = AuthRuntime::default();
    let err = runtime
        .prepare_sign_context(
            &AuthRequestMeta {
                venue: "x".into(),
                surface: AuthSurface::PrivateRest,
                auth_mode: AuthMode::HmacHeader,
                requires_auth: true,
                request_name: "place_order".into(),
                key_id: None,
            },
            "POST",
            "/private",
            "",
            "",
            1,
            Some(1),
            Some("i".into()),
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
}

#[test]
fn wrong_mode_fails_when_auth_required() {
    let runtime = AuthRuntime::default();
    let err = runtime
        .prepare_sign_context(
            &AuthRequestMeta {
                venue: "x".into(),
                surface: AuthSurface::PrivateRest,
                auth_mode: AuthMode::None,
                requires_auth: true,
                request_name: "place_order".into(),
                key_id: Some("k1".into()),
            },
            "POST",
            "/private",
            "",
            "",
            1,
            Some(1),
            None,
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
}

#[test]
fn mode_specific_material_validation_is_enforced() {
    let runtime = AuthRuntime::default();
    let meta = AuthRequestMeta {
        venue: "x".into(),
        surface: AuthSurface::PrivateWs,
        auth_mode: AuthMode::SessionToken,
        requires_auth: true,
        request_name: "ws_login".into(),
        key_id: Some("k1".into()),
    };
    let err = runtime
        .enforce_auth_material_presence(&meta, &AuthMaterial::default())
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::MissingAuth);
}
