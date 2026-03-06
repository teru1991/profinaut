use ucel_core::{AuthSurface, ErrorCode, NonceScope};
use ucel_testkit::auth::MonotonicNonceProvider;
use ucel_transport::auth::{AuthRuntime, NonceProvider};

#[test]
fn nonce_is_monotonic_per_scope() {
    let provider = MonotonicNonceProvider::default();
    let scope = NonceScope {
        venue: "bitbank".into(),
        key_id: "k1".into(),
        surface: AuthSurface::PrivateRest,
    };

    let a = provider.next_nonce(&scope).unwrap();
    let b = provider.next_nonce(&scope).unwrap();
    assert!(b > a);
}

#[test]
fn nonce_scopes_are_isolated() {
    let provider = MonotonicNonceProvider::default();
    let scope_a = NonceScope {
        venue: "bitbank".into(),
        key_id: "k1".into(),
        surface: AuthSurface::PrivateRest,
    };
    let scope_b = NonceScope {
        venue: "bitbank".into(),
        key_id: "k2".into(),
        surface: AuthSurface::PrivateRest,
    };

    assert_eq!(provider.next_nonce(&scope_a).unwrap(), 1);
    assert_eq!(provider.next_nonce(&scope_b).unwrap(), 1);
}

#[test]
fn clock_offset_updates_with_monotonic_observed_time() {
    let runtime = AuthRuntime::default();
    let first = runtime
        .update_server_time_offset(None, 1_000, 900, 100)
        .unwrap();
    let second = runtime
        .update_server_time_offset(Some(first), 1_100, 1_000, 101)
        .unwrap();
    assert_eq!(second.offset_ms, 100);
}

#[test]
fn abnormal_clock_jump_is_error() {
    let runtime = AuthRuntime {
        max_clock_skew_ms: 100,
    };
    let err = runtime
        .update_server_time_offset(None, 2_000, 1_000, 100)
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidOrder);
}
