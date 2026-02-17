//! Health endpoint scaffolding (`/healthz`).
//!
//! Reports service status, config load state, and per-instance descriptor
//! validation results.

use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub service: &'static str,
    pub version: &'static str,
    pub config_loaded: bool,
    pub config_error: Option<String>,
    pub descriptors_loaded_count: usize,
    pub instances: Vec<InstanceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceStatus {
    pub name: String,
    pub enabled: bool,
    pub descriptor_path: String,
    pub descriptor_name: Option<String>,
    pub descriptor_version: Option<String>,
    pub validation_status: &'static str,
    pub error_message: Option<String>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn healthz(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let descriptors_loaded_count = state
        .instances
        .iter()
        .filter(|i| i.enabled && i.validation_status == "OK")
        .count();

    Json(HealthResponse {
        service: "crypto-collector",
        version: "v1.4",
        config_loaded: state.config_loaded,
        config_error: state.config_error.clone(),
        descriptors_loaded_count,
        instances: state.instances.clone(),
    })
}
