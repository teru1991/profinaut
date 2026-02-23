use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod config;
mod lock;
mod adapters;
mod supervisor;

use config::IngestConfig;
use supervisor::{run_supervisor, SupervisorShutdown};

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
            error!(err=%e, "config load failed");
            std::process::exit(2);
        }
    };

    info!(?cfg, "ucel-ws-subscriber starting");

    // prevent double-start with same store/journal
    let lock_path = cfg.store_path.with_extension("lock");
    let _pid_lock = match lock::PidLock::acquire(&lock_path) {
        Ok(l) => l,
        Err(e) => {
            error!(err=%e, "pid lock acquire failed");
            std::process::exit(3);
        }
    };
    let shutdown = SupervisorShutdown::new();

    // SIGINT/SIGTERM
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

    if let Err(e) = run_supervisor(&cfg, shutdown.clone()).await {
        error!(err=%e, "supervisor failed");
        std::process::exit(1);
    }

    // wait until shutdown
    while !shutdown.is_triggered() {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    info!("ucel-ws-subscriber exiting");
}
