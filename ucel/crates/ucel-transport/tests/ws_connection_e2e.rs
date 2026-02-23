use std::net::SocketAddr;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;

use ucel_subscription_planner::{canon_params, stable_key, SubscriptionKey};
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
        Ok(vec!["BTC/JPY".to_string()])
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        _params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        Ok(vec![OutboundMsg {
            text: json!({
                "command":"subscribe",
                "op_id": op_id,
                "symbol": symbol
            })
            .to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };
        let op_id = v
            .get("op_id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        let symbol = v
            .get("symbol")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        InboundClass::Data {
            op_id,
            symbol,
            params_canon_hint: Some("{}".to_string()),
        }
    }
}

/// 疑似WSサーバ
/// - first_connection_close: 最初の接続で subscribe を受け取ったら close する（reconnect誘発）
/// - send_oversized: 接続直後に巨大フレームを送る（Stop誘発）
async fn spawn_fake_ws_server(
    first_connection_close: bool,
    send_oversized: bool,
    oversized_bytes: usize,
) -> (SocketAddr, oneshot::Sender<()>, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let received2 = received.clone();

    tokio::spawn(async move {
        let mut conn_count = 0usize;

        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    break;
                }
                accept = listener.accept() => {
                    let (stream, _) = accept.unwrap();
                    conn_count += 1;
                    let current_conn = conn_count;

                    let received = received2.clone();
                    tokio::spawn(async move {
                        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut w, mut r) = ws.split();

                        if send_oversized {
                            // 受信ループ開始前に巨大フレーム
                            let big = "X".repeat(oversized_bytes);
                            let _ = w.send(Message::Text(big.into())).await;
                            let _ = w.send(Message::Close(None)).await;
                            return;
                        }

                        // subscribe を1回受け取るまで待つ
                        if let Some(Ok(Message::Text(t))) = r.next().await {
                            received.lock().await.push(t.to_string());

                            // Data を返す（Active確定用）
                            let _ = w
                                .send(Message::Text(
                                    json!({
                                        "op_id": "crypto.public.ws.ticker.update",
                                        "symbol": "BTC/JPY"
                                    })
                                    .to_string()
                                    .into(),
                                ))
                                .await;

                            // 1回目の接続なら close して reconnect を誘発
                            if first_connection_close && current_conn == 1 {
                                let _ = w.send(Message::Close(None)).await;
                                return;
                            }

                            // 2回目以降はもう一発 Data を返して継続
                            let _ = w
                                .send(Message::Text(
                                    json!({
                                        "op_id": "crypto.public.ws.ticker.update",
                                        "symbol": "BTC/JPY"
                                    })
                                    .to_string()
                                    .into(),
                                ))
                                .await;
                            tokio::time::sleep(Duration::from_millis(200)).await;
                        }
                    });
                }
            }
        }
    });

    (addr, stop_tx, received)
}

fn write_rules_toml(dir: &std::path::Path, exchange_id: &str, mps: u32) {
    std::fs::create_dir_all(dir).unwrap();
    let toml = format!(
        r#"
exchange_id = "{exchange_id}"
support_level = "full"

[rate]
messages_per_second = {mps}
messages_per_hour = 3600

[heartbeat]
ping_interval_secs = 5
idle_timeout_secs = 2

[safety_profile]
max_streams_per_conn = 5
max_symbols_per_conn = 5
"#
    );
    std::fs::write(dir.join(format!("{exchange_id}.toml")), toml).unwrap();
}

#[tokio::test]
async fn e2e_reconnect_drip_wal() {
    // fake server: first connection closes after first subscribe => reconnect
    let (addr, stop_tx, received) = spawn_fake_ws_server(true, false, 0).await;

    // rules
    let tmp = tempfile::tempdir().unwrap();
    let rules_dir = tmp.path().join("rules");
    write_rules_toml(&rules_dir, "gmocoin", 10);
    let rules = load_for_exchange(&rules_dir, "gmocoin");

    // WAL
    let wal_dir = tmp.path().join("wal");
    let wal = ucel_journal::WalWriter::open(
        &wal_dir,
        64 * 1024 * 1024,
        ucel_journal::FsyncMode::Balanced,
    )
    .map_err(|e| format!("{e}"))
    .unwrap();
    let wal = Arc::new(Mutex::new(wal));

    // store seed (stable key)
    let mut store = SubscriptionStore::open(":memory:").unwrap();
    let k = SubscriptionKey {
        exchange_id: "gmocoin".to_string(),
        op_id: "crypto.public.ws.ticker.update".to_string(),
        symbol: Some("BTC/JPY".to_string()),
        params: json!({}),
    };
    let key = stable_key(&k);
    store
        .seed(
            &[SubscriptionRow {
                key: key.clone(),
                exchange_id: "gmocoin".to_string(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_json: canon_params(&k.params),
                assigned_conn: Some("gmocoin-conn-1".to_string()),
            }],
            1,
        )
        .unwrap();

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(TestAdapter {
        exchange_id: "gmocoin".to_string(),
        url: format!("ws://{}", addr),
    });

    let shutdown = ShutdownToken {
        flag: Arc::new(AtomicBool::new(false)),
    };
    let cfg = WsRunConfig {
        exchange_id: "gmocoin".to_string(),
        conn_id: "gmocoin-conn-1".to_string(),
        recv_queue_cap: 256,
        max_frame_bytes: 1024 * 1024,
        max_inflight_per_conn: 10,
        connect_timeout: Duration::from_secs(2),
        idle_timeout: Duration::from_secs(2),
        reconnect_storm_window: Duration::from_secs(30),
        reconnect_storm_max: 10,
    };

    // run in background, then stop
    let run = tokio::spawn(async move {
        run_ws_connection(adapter, rules, &mut store, wal, cfg, shutdown).await
    });

    tokio::time::sleep(Duration::from_millis(800)).await;
    // stop server + cancel task（connectionはreconnectを続けるのでここで止める）
    let _ = stop_tx.send(());

    // received at least one subscribe
    let r = received.lock().await.clone();
    assert!(!r.is_empty(), "server should receive subscribe");
    // WAL should have file
    // （ucel-journal の出力ファイル名は実装依存なので “ディレクトリに何か増えた” を見る）
    let wal_files: Vec<_> = std::fs::read_dir(tmp.path().join("wal")).unwrap().collect();
    assert!(!wal_files.is_empty(), "wal dir should have files");

    run.abort();
}

#[tokio::test]
async fn e2e_stop_on_oversized_frame() {
    let (addr, stop_tx, _received) = spawn_fake_ws_server(false, true, 2 * 1024 * 1024).await;

    let tmp = tempfile::tempdir().unwrap();
    let rules_dir = tmp.path().join("rules");
    write_rules_toml(&rules_dir, "gmocoin", 10);
    let rules = load_for_exchange(&rules_dir, "gmocoin");

    let wal_dir = tmp.path().join("wal");
    let wal = ucel_journal::WalWriter::open(
        &wal_dir,
        64 * 1024 * 1024,
        ucel_journal::FsyncMode::Balanced,
    )
    .map_err(|e| format!("{e}"))
    .unwrap();
    let wal = Arc::new(Mutex::new(wal));

    let mut store = SubscriptionStore::open(":memory:").unwrap();

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(TestAdapter {
        exchange_id: "gmocoin".to_string(),
        url: format!("ws://{}", addr),
    });

    let shutdown = ShutdownToken {
        flag: Arc::new(AtomicBool::new(false)),
    };
    let cfg = WsRunConfig {
        exchange_id: "gmocoin".to_string(),
        conn_id: "gmocoin-conn-1".to_string(),
        recv_queue_cap: 64,
        max_frame_bytes: 1024, // small => trigger stop
        max_inflight_per_conn: 1,
        connect_timeout: Duration::from_secs(2),
        idle_timeout: Duration::from_secs(2),
        reconnect_storm_window: Duration::from_secs(30),
        reconnect_storm_max: 10,
    };

    let res = run_ws_connection(adapter, rules, &mut store, wal, cfg, shutdown).await;
    assert!(res.is_err(), "oversized frame should stop");

    let _ = stop_tx.send(());
}

#[tokio::test]
async fn e2e_symbol_less_subscription_is_dripped() {
    let (addr, stop_tx, received) = spawn_fake_ws_server(false, false, 0).await;

    let tmp = tempfile::tempdir().unwrap();
    let rules_dir = tmp.path().join("rules");
    write_rules_toml(&rules_dir, "gmocoin", 10);
    let rules = load_for_exchange(&rules_dir, "gmocoin");

    let wal_dir = tmp.path().join("wal");
    let wal = ucel_journal::WalWriter::open(
        &wal_dir,
        64 * 1024 * 1024,
        ucel_journal::FsyncMode::Balanced,
    )
    .map_err(|e| format!("{e}"))
    .unwrap();
    let wal = Arc::new(Mutex::new(wal));

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    let k = SubscriptionKey {
        exchange_id: "gmocoin".to_string(),
        op_id: "bybit.public.ws.insurance".to_string(),
        symbol: None,
        params: json!({}),
    };
    let key = stable_key(&k);
    store
        .seed(
            &[SubscriptionRow {
                key,
                exchange_id: "gmocoin".to_string(),
                op_id: k.op_id.clone(),
                symbol: None,
                params_json: canon_params(&k.params),
                assigned_conn: Some("gmocoin-conn-1".to_string()),
            }],
            1,
        )
        .unwrap();

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(TestAdapter {
        exchange_id: "gmocoin".to_string(),
        url: format!("ws://{}", addr),
    });

    let shutdown = ShutdownToken {
        flag: Arc::new(AtomicBool::new(false)),
    };
    let cfg = WsRunConfig {
        exchange_id: "gmocoin".to_string(),
        conn_id: "gmocoin-conn-1".to_string(),
        recv_queue_cap: 256,
        max_frame_bytes: 1024 * 1024,
        max_inflight_per_conn: 10,
        connect_timeout: Duration::from_secs(2),
        idle_timeout: Duration::from_secs(2),
        reconnect_storm_window: Duration::from_secs(30),
        reconnect_storm_max: 10,
    };

    let run = tokio::spawn(async move {
        run_ws_connection(adapter, rules, &mut store, wal, cfg, shutdown).await
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = stop_tx.send(());

    let r = received.lock().await.clone();
    assert!(
        r.iter().any(|m| m.contains("\"symbol\":\"\"")),
        "server should receive subscribe with empty symbol for symbol-less key"
    );

    run.abort();
}
