//! Multi-Exchange Market Data Collector Framework v1.4 — Crypto Subsystem
//!
//! Service skeleton: Tokio runtime, structured tracing, graceful shutdown,
//! config loading, descriptor validation, and `/healthz` endpoint.

mod config;
mod descriptor;
pub mod dsl;
pub mod engine;
mod health;
pub mod json_pointer;
pub mod maps;
pub mod mini_expr;
pub mod placeholder;
mod state;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{routing::get, Router};
use tracing::{error, info, warn};

use config::CollectorConfig;
use health::InstanceStatus;
use state::AppState;

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // Initialize tracing (structured JSON logging).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    info!("crypto-collector v1.4 starting");

    // Determine config path from CLI arg or default.
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "collector.toml".to_string());

    let state = build_state(&config_path);

    let http_port = resolve_http_port(&config_path);

    let app_state = Arc::new(state);
    let app = Router::new()
        .route("/healthz", get(health::healthz))
        .with_state(app_state);

    let addr: SocketAddr = ([0, 0, 0, 0], http_port).into();
    info!(%addr, "binding HTTP server");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(%e, "failed to bind");
            std::process::exit(1);
        }
    };

    info!("crypto-collector listening on http://{addr}");

    // Graceful shutdown: wait for SIGINT (Ctrl+C) or SIGTERM.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| {
            error!(%e, "server error");
            std::process::exit(1);
        });

    info!("crypto-collector shut down gracefully");
}

// ---------------------------------------------------------------------------
// State construction
// ---------------------------------------------------------------------------

/// Build the application state by loading config and validating descriptors.
/// Never panics — errors are captured in the state for `/healthz` reporting.
fn build_state(config_path: &str) -> AppState {
    let cfg_result = config::load_config(Path::new(config_path));

    match cfg_result {
        Ok(cfg) => {
            info!(exchanges = cfg.exchanges.len(), "config loaded");
            let instances = load_descriptors(&cfg, config_path);
            AppState {
                config_loaded: true,
                config_error: None,
                instances,
            }
        }
        Err(e) => {
            error!(%e, "config load failed");
            AppState {
                config_loaded: false,
                config_error: Some(e.to_string()),
                instances: Vec::new(),
            }
        }
    }
}

/// Resolve the HTTP port: try to read from config, fall back to 8090.
fn resolve_http_port(config_path: &str) -> u16 {
    match config::load_config(Path::new(config_path)) {
        Ok(cfg) => cfg.run.http_port,
        Err(_) => {
            warn!("could not read http_port from config; defaulting to 8090");
            8090
        }
    }
}

/// Load and validate descriptors for each exchange instance.
fn load_descriptors(cfg: &CollectorConfig, config_path: &str) -> Vec<InstanceStatus> {
    let config_dir = Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    cfg.exchanges
        .iter()
        .map(|inst| {
            let desc_path = resolve_descriptor_path(config_dir, &inst.descriptor_path);
            let desc_path_str = desc_path.display().to_string();

            if !inst.enabled {
                info!(exchange = %inst.name, "skipping descriptor load (disabled)");
                return InstanceStatus {
                    name: inst.name.clone(),
                    enabled: false,
                    descriptor_path: desc_path_str,
                    descriptor_name: None,
                    descriptor_version: None,
                    validation_status: "OK",
                    error_message: None,
                };
            }

            match descriptor::load_descriptor(&desc_path) {
                Ok(desc) => {
                    info!(exchange = %inst.name, descriptor = %desc.meta.name, "descriptor loaded");

                    // Warn about symbol_map_file if present but not found
                    if let Some(ref maps) = desc.maps {
                        if let Some(ref map_file) = maps.symbol_map_file {
                            let map_path = resolve_descriptor_path(config_dir, map_file);
                            if !map_path.exists() {
                                warn!(
                                    exchange = %inst.name,
                                    path = %map_path.display(),
                                    "symbol_map_file not found (warning only)"
                                );
                            }
                        }
                    }

                    InstanceStatus {
                        name: inst.name.clone(),
                        enabled: true,
                        descriptor_path: desc_path_str,
                        descriptor_name: Some(desc.meta.name.clone()),
                        descriptor_version: Some(desc.meta.version.clone()),
                        validation_status: "OK",
                        error_message: None,
                    }
                }
                Err(e) => {
                    error!(exchange = %inst.name, %e, "descriptor validation failed");
                    InstanceStatus {
                        name: inst.name.clone(),
                        enabled: true,
                        descriptor_path: desc_path_str,
                        descriptor_name: None,
                        descriptor_version: None,
                        validation_status: "ERROR",
                        error_message: Some(e.to_string()),
                    }
                }
            }
        })
        .collect()
}

/// Resolve a descriptor path relative to the config file's directory.
fn resolve_descriptor_path(config_dir: &Path, descriptor_path: &str) -> PathBuf {
    let p = Path::new(descriptor_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        config_dir.join(p)
    }
}

// ---------------------------------------------------------------------------
// Graceful shutdown
// ---------------------------------------------------------------------------

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { info!("received SIGINT, shutting down"); }
        () = terminate => { info!("received SIGTERM, shutting down"); }
    }
}
