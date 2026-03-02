use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio::task::LocalSet;
use tokio_tungstenite::tungstenite::Message;

use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::load_for_exchange;

#[derive(Clone)]
struct TestAdapter {
    exchange_id: String,
    url: String,
}

#[async_trait::async_trait]
impl WsVenueAdapter for TestAdapter {
    fn exchange_id(&self) -> &str {
        &self.exchange_id
    }

    fn ws_url(&self) -> String {
        self.url.clone()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        _params: &serde_json::Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        Ok(vec![OutboundMsg {
            text: json!({"command":"subscribe","op_id":op_id,"symbol":symbol}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: serde_json::Value = serde_json::from_slice(raw).unwrap_or(json!({}));
        let kind = v.get("kind").and_then(|x| x.as_str()).unwrap_or("");
        if kind == "nack" {
            return InboundClass::Nack {
                reason: v
                    .get("reason")
                    .and_then(|x| x.as_str())
                    .unwrap_or("rate limit")
                    .to_string(),
                op_id: v
                    .get("op_id")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                symbol: v
                    .get("symbol")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                params_canon_hint: Some("{}".to_string()),
                retry_after_ms: v.get("retry_after_ms").and_then(|x| x.as_u64()),
            };
        }
        InboundClass::Unknown
    }
}

async fn spawn_ws_server_send_nack() -> (std::net::SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                acc = listener.accept() => {
                    let (stream, _) = acc.unwrap();
                    tokio::spawn(async move {
                        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut w, mut r) = ws.split();
                        if let Some(Ok(Message::Text(_t))) = r.next().await {
                            let _ = w.send(Message::Text(json!({
                                "kind":"nack",
                                "reason":"rate limit",
                                "op_id":"crypto.private.orders",
                                "retry_after_ms": 800
                            }).to_string())).await;
                            let _ = w.send(Message::Close(None)).await;
                        }
                    });
                }
            }
        }
    });

    (addr, stop_tx)
}

fn write_rules_toml(dir: &std::path::Path, exchange_id: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let toml = format!(
        r#"
exchange_id = "{exchange_id}"
support_level = "full"

[rate]
messages_per_second = 10
messages_per_hour = 3600

[heartbeat]
ping_interval_secs = 0
idle_timeout_secs = 2

[safety_profile]
max_streams_per_conn = 5
max_symbols_per_conn = 5
"#
    );
    std::fs::write(dir.join(format!("{exchange_id}.toml")), toml).unwrap();
}

#[tokio::test]
async fn ws_nack_rate_limit_sets_cooldown() {
    let (addr, stop_tx) = spawn_ws_server_send_nack().await;

    let tmp = tempfile::tempdir().unwrap();
    let rules_dir = tmp.path().join("rules");
    write_rules_toml(&rules_dir, "gmocoin");
    let rules = load_for_exchange(&rules_dir, "gmocoin");

    let wal_dir = tmp.path().join("wal");
    let wal = ucel_journal::WalWriter::open(
        &wal_dir,
        64 * 1024 * 1024,
        ucel_journal::FsyncMode::Balanced,
    )
    .unwrap();
    let wal = Arc::new(Mutex::new(wal));

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    let key = "gmocoin|crypto.private.orders||{}";
    store
        .seed(
            &[SubscriptionRow {
                key: key.to_string(),
                exchange_id: "gmocoin".to_string(),
                op_id: "crypto.private.orders".to_string(),
                symbol: None,
                params_json: "{}".to_string(),
                assigned_conn: Some("gmocoin-conn-1".to_string()),
            }],
            1,
        )
        .unwrap();

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(TestAdapter {
        exchange_id: "gmocoin".to_string(),
        url: format!("ws://{addr}"),
    });

    let flag = Arc::new(AtomicBool::new(false));
    let shutdown = ShutdownToken { flag: flag.clone() };
    let cfg = WsRunConfig {
        exchange_id: "gmocoin".to_string(),
        conn_id: "gmocoin-conn-1".to_string(),
        rl_default_penalty_ms: 500,
        ..Default::default()
    };

    let local = LocalSet::new();
    let mut store = local
        .run_until(async move {
            tokio::task::spawn_local(async move {
                tokio::time::sleep(Duration::from_millis(700)).await;
                flag.store(true, Ordering::Relaxed);
                let _ = stop_tx.send(());
            });
            let _ = run_ws_connection(adapter, rules, &mut store, wal, cfg, shutdown).await;
            store
        })
        .await;

    let until = store.rate_limit_until_of(key).unwrap();
    assert!(until.is_some());
    let attempts = store.attempts_of(key).unwrap().unwrap_or(0);
    assert!(attempts >= 1);

    let got = store
        .next_pending_batch("gmocoin", "gmocoin-conn-1", 10, 1)
        .unwrap();
    assert_eq!(got.len(), 0);
}
