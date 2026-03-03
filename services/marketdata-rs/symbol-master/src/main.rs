use std::sync::Arc;

use symbol_master::app::{AppState, Supervisor};
use symbol_master::config::AppConfig;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mut config_path: Option<std::path::PathBuf> = None;
    while let Some(arg) = args.next() {
        if arg == "--config" {
            config_path = args.next().map(std::path::PathBuf::from);
        }
    }

    let cfg_path = config_path.unwrap_or_else(|| {
        std::path::PathBuf::from("services/marketdata-rs/symbol-master/config.yaml")
    });

    let cfg = match AppConfig::load_yaml(&cfg_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("symbol-master: config error: {e}");
            std::process::exit(2);
        }
    };

    let app_state = AppState::new(cfg.clone());
    let sup = Arc::new(Supervisor::new());

    let resync = app_state.resync.clone();
    let resync_h = tokio::spawn(resync.run());

    let listen = cfg.http.listen.clone();
    let router = symbol_master::http::router(app_state.clone());
    let http_h = tokio::spawn(async move {
        let addr: std::net::SocketAddr = listen.parse().expect("http.listen must be SocketAddr");
        let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
        axum::serve(listener, router).await.expect("serve");
    });

    sup.clone().spawn_workers(app_state).await;

    let _ = tokio::signal::ctrl_c().await;

    sup.shutdown().await;
    http_h.abort();
    resync_h.abort();
}
