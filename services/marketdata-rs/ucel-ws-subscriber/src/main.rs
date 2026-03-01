use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod adapters;
mod config;
mod http;
mod lock;
mod state;
mod supervisor;

use config::IngestConfig;
use state::AppState;
use supervisor::{run_supervisor, SupervisorShutdown};
use tokio::task::LocalSet;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ucel_ws_subscriber=info")),
        )
        .init();

    let cfg = match IngestConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(err = %e, "config load failed");
            std::process::exit(2);
        }
    };

    info!(?cfg, "ucel-ws-subscriber starting");

    let lock_path = cfg.store_path.with_extension("lock");
    let _pid_lock = match lock::PidLock::acquire(&lock_path) {
        Ok(l) => l,
        Err(e) => {
            error!(err = %e, "pid lock acquire failed");
            std::process::exit(3);
        }
    };
    let shutdown = SupervisorShutdown::new();

    {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).expect("sigterm");
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = sigterm.recv() => {},
                }
            }
            #[cfg(not(unix))]
            {
                let _ = tokio::signal::ctrl_c().await;
            }
            warn!("shutdown signal received");
            shutdown.trigger();
        });
    }

    let app_state = AppState::new(
        cfg.exchange_allowlist
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        std::env::var("UCEL_CONN_ID").unwrap_or_else(|_| "conn-1".to_string()),
    );

    let bind =
        std::env::var("UCEL_WS_SUB_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let router = crate::http::router(app_state.clone());
    let http_shutdown = shutdown.clone();
    let http_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&bind)
            .await
            .map_err(|e| e.to_string())?;
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                while !http_shutdown.is_triggered() {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            })
            .await
            .map_err(|e| e.to_string())
    });

    let sup_task =
        tokio::spawn(async move { run_supervisor(&cfg, shutdown.clone(), app_state).await });

    let (http_res, sup_res) = tokio::join!(http_task, sup_task);
    match http_res {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error!(err=%e, "http task failed");
            std::process::exit(1);
        }
        Err(e) => {
            error!(err=%e, "http task join failed");
            std::process::exit(1);
        }
    }

    match sup_res {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error!(err=%e, "supervisor failed");
            std::process::exit(1);
        }
        Err(e) => {
            error!(err=%e, "supervisor task join failed");
            std::process::exit(1);
        }
    }

    info!("ucel-ws-subscriber exiting");
}
