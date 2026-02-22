use std::path::PathBuf;

use ucel_ws_subscriber::config::IngestConfig;
use ucel_ws_subscriber::supervisor::{run, SupervisorShutdown};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = IngestConfig::default();
    let shutdown = SupervisorShutdown::new();
    let shutdown_signal = shutdown.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown_signal.trigger();
    });

    let coverage_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../ucel/coverage");
    let rules_dir = PathBuf::from(&cfg.rules_dir);
    let store_path = PathBuf::from(&cfg.store_path);
    let journal_dir = PathBuf::from(&cfg.journal_dir);

    run(
        &coverage_dir,
        &rules_dir,
        &store_path,
        &journal_dir,
        shutdown,
    )
    .await
    .expect("supervisor run");
}
