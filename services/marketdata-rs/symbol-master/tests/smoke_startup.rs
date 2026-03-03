use axum::body::Body;
use axum::http::{Request, StatusCode};
use symbol_master::config::{AppConfig, ExchangeConfig, ExchangeMode, HttpConfig};
use tower::ServiceExt;

#[tokio::test]
async fn healthz_works() {
    let cfg = AppConfig {
        http: HttpConfig {
            listen: "127.0.0.1:0".to_string(),
        },
        exchanges: vec![ExchangeConfig {
            exchange_id: "gmo".to_string(),
            mode: ExchangeMode::PublicOnly,
            params: serde_yaml::Value::Null,
        }],
    };

    let app = symbol_master::app::AppState::new(cfg);
    let router = symbol_master::http::router(app);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(matches!(
        response.status(),
        StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE
    ));
}
