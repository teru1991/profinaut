use crate::app::{AppState, HealthStatus};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

#[derive(Clone)]
pub struct HttpState {
    pub app: AppState,
}

#[derive(Serialize)]
struct HealthzBody {
    status: &'static str,
    reason: Option<&'static str>,
}

async fn healthz(State(st): State<HttpState>) -> impl IntoResponse {
    let snap = st.app.health_rx.borrow().clone();
    match snap.status {
        HealthStatus::Ok => (
            StatusCode::OK,
            Json(HealthzBody {
                status: "ok",
                reason: None,
            }),
        ),
        HealthStatus::Degraded { reason } => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthzBody {
                status: "degraded",
                reason: Some(reason),
            }),
        ),
    }
}

async fn readyz(State(st): State<HttpState>) -> impl IntoResponse {
    let snap = st.app.health_rx.borrow().clone();
    match snap.status {
        HealthStatus::Ok => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub fn router(app: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(HttpState { app })
}
