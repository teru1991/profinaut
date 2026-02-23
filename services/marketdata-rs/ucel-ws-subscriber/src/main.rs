use ucel_ws_subscriber::config::IngestConfig;
use ucel_ws_subscriber::supervisor::{run_supervisor, SupervisorShutdown};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = IngestConfig::default();
    let shutdown = SupervisorShutdown::new();
    let shutdown_signal = shutdown.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown_signal.trigger();
    });

    run_supervisor(&cfg, shutdown).await.expect("supervisor run");
}
