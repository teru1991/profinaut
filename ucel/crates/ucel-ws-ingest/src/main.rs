use ucel_ws_ingest::{config::IngestConfig, supervisor::run_supervisor};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = IngestConfig::default();
    let reports = run_supervisor(&cfg).await.expect("supervisor startup");
    println!("ucel-ws-ingest running for {} exchanges", reports.len());
}
