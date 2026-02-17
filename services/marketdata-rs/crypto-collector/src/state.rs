//! Shared application state built during startup.

use crate::health::InstanceStatus;

/// Immutable application state shared with HTTP handlers via `Arc<AppState>`.
#[derive(Debug)]
pub struct AppState {
    pub config_loaded: bool,
    pub config_error: Option<String>,
    pub instances: Vec<InstanceStatus>,
}
