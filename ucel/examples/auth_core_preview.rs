use std::collections::HashMap;

use ucel_core::{AuthMaterial, AuthMode, AuthRequestMeta, AuthSurface, SecretRef};
use ucel_testkit::auth::{
    EchoSigner, MonotonicNonceProvider, SequenceIdempotencyProvider, StaticSecretResolver,
};
use ucel_transport::auth::AuthRuntime;

fn main() {
    let mut material_by_key = HashMap::new();
    material_by_key.insert(
        "demo-key".to_string(),
        AuthMaterial {
            api_key: Some("demo-api-key".to_string()),
            api_secret: Some("demo-secret".to_string()),
            ..Default::default()
        },
    );

    let runtime = AuthRuntime::default();
    let parts = runtime
        .resolve_and_sign(
            &StaticSecretResolver { material_by_key },
            &MonotonicNonceProvider::default(),
            &SequenceIdempotencyProvider::default(),
            &EchoSigner,
            &SecretRef {
                key_id: Some("demo-key".into()),
                alias: None,
            },
            &AuthRequestMeta {
                venue: "bitbank".into(),
                surface: AuthSurface::PrivateRest,
                auth_mode: AuthMode::HmacHeader,
                requires_auth: true,
                request_name: "place_order".into(),
                key_id: Some("demo-key".into()),
            },
            "POST",
            "/private/order",
            "symbol=BTC_JPY",
            "{\"qty\":1}",
            1_700_000_000,
        )
        .expect("auth core preview should build signed parts");

    println!("signed headers keys: {:?}", parts.headers.keys().collect::<Vec<_>>());
}
