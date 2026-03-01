use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use ucel_transport::diagnostics::support_bundle::{build_support_bundle, SupportBundleInput};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/support_bundle", get(support_bundle))
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let h = state.health.read().clone();
    Json(h)
}

async fn support_bundle(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.health.read().clone();
    let rules = state.rules_snapshot.read().clone();

    let bundle = build_support_bundle(SupportBundleInput {
        exchange_id: state.exchange_id.clone(),
        conn_id: state.conn_id.clone(),
        health,
        metrics: state.metrics.clone(),
        events: state.events.clone(),
        rules_snapshot: rules,
    });
    Json(bundle)
}
