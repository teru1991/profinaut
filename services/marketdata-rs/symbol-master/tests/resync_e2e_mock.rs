use std::time::Duration;

use axum::{routing::get, Json, Router};
use serde_json::json;
use symbol_master::resync_loop::ResyncCoordinator;
use tempfile::tempdir;
use tokio::sync::watch;
use ucel_symbol_adapter::ResyncHint;
use ucel_symbol_store::SymbolStore;

#[tokio::test]
async fn resync_hint_fetches_snapshot_updates_store_and_writes_checkpoint() {
    let app = Router::new().route(
        "/snapshot",
        get(|| async {
            Json(json!({
                "schema_version": 1,
                "instruments": [{
                    "exchange": "gmocoin",
                    "market_type": "spot",
                    "raw_symbol": "BTC_JPY",
                    "base": "BTC",
                    "quote": "JPY",
                    "status": "trading",
                    "tick_size": "1",
                    "lot_size": "0.01",
                    "meta": {"source":"mock"}
                }]
            }))
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let dir = tempdir().unwrap();
    let checkpoint_path = dir.path().join("checkpoints.jsonl");
    let store = std::sync::Arc::new(SymbolStore::new());
    let coordinator = std::sync::Arc::new(ResyncCoordinator::new(
        store.clone(),
        checkpoint_path.clone(),
    ));

    let (tx, rx) = watch::channel(None::<ResyncHint>);
    coordinator
        .register_exchange("gmocoin", rx, Some(format!("http://{addr}/snapshot")))
        .await;

    let run_h = tokio::spawn(coordinator.clone().run());
    tx.send(Some(ResyncHint::Lagged { reason: "test" }))
        .unwrap();

    let mut ok = false;
    for _ in 0..20 {
        if !store.snapshot().instruments.is_empty() {
            ok = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let st = coordinator.snapshot().await;
    assert!(
        ok,
        "store should be updated by resync path, last_error={:?}",
        st.last_error
    );

    let cp = std::fs::read_to_string(&checkpoint_path).unwrap();
    assert!(cp.contains("\"exchange_id\":\"gmocoin\""));

    run_h.abort();
    server.abort();
}
