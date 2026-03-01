use axum::body::{to_bytes, Body};
use http::Request;
use tower::util::ServiceExt;

use ucel_ws_subscriber::http::router;
use ucel_ws_subscriber::state::AppState;

#[tokio::test]
async fn endpoints_return_json() {
    let st = AppState::new("x".into(), "c".into());
    let app = router(st);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.get("status").is_some());

    let resp2 = app
        .oneshot(
            Request::builder()
                .uri("/support_bundle")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp2.status(), http::StatusCode::OK);
    let body2 = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let v2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    assert_eq!(v2.get("version").and_then(|x| x.as_i64()), Some(1));
}
