use ucel_ws_subscriber::{config::IngestConfig, supervisor::run_supervisor};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = IngestConfig::default();
    let exchanges = run_supervisor(&cfg).await.expect("supervisor startup");
    println!("ucel-ws-subscriber running for {} exchanges", exchanges.len());
}
