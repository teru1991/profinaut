use crate::config::IngestConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use base64::Engine as _;
use ucel_journal::{FsyncMode, RawRecord, WalWriter};
use ucel_subscription_planner::{generate_plan, load_all_ws_ops};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::connection::open as ws_open;
use ucel_transport::ws::reconnect::backoff_with_jitter_ms;
use ucel_ws_rules::{load_for_exchange, SupportLevel};

fn should_include_op(op: &str, enable_private_ws: bool) -> bool {
    op.starts_with("crypto.public.ws.") || (enable_private_ws && op.starts_with("crypto.private.ws."))
}

fn subscription_key(exchange_id: &str, op_id: &str, symbol: Option<&str>) -> String {
    format!("{exchange_id}:{op_id}:{}", symbol.unwrap_or_default())
}

pub async fn run_supervisor(cfg: &IngestConfig) -> Result<Vec<String>, String> {
    let coverage_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage");
    let coverage = load_all_ws_ops(&coverage_dir)?;
    let mut started = Vec::new();

    for (exchange, ws_ops) in coverage {
        if let Some(allow) = &cfg.exchange_allowlist {
            if !allow.contains(&exchange) {
                continue;
            }
        }
        if exchange == "sbivc" {
            // unsupported in v1 baseline
            continue;
        }

        let rules = load_for_exchange(std::path::Path::new(&cfg.rules_dir), &exchange);
        if matches!(rules.support_level, SupportLevel::NotSupported) {
            continue;
        }
        let ws_ops: Vec<String> = ws_ops
            .into_iter()
            .filter(|op| should_include_op(op, cfg.enable_private_ws))
            .collect();
        if ws_ops.is_empty() {
            continue;
        }

        let symbols = vec!["BTC/USDT".to_string()];
        let plan = generate_plan(&exchange, &ws_ops, &symbols, &rules);
        let conn_by_key: HashMap<String, String> = plan
            .conn_plans
            .iter()
            .flat_map(|cp| cp.keys.iter().map(|k| (k.clone(), cp.conn_id.clone())))
            .collect();

        let mut store = SubscriptionStore::open(&cfg.store_path)?;
        let rows: Vec<SubscriptionRow> = plan
            .seed
            .iter()
            .map(|k| SubscriptionRow {
                key: subscription_key(&k.exchange_id, &k.op_id, k.symbol.as_deref()),
                exchange_id: k.exchange_id.clone(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_json: k.params.to_string(),
                assigned_conn: conn_by_key
                    .get(&subscription_key(&k.exchange_id, &k.op_id, k.symbol.as_deref()))
                    .cloned()
                    .unwrap_or_else(|| format!("{}-conn-1", exchange))
                    .into(),
            })
            .collect();
        store.seed(&rows, 0)?;
        started.push(exchange);
    }

    Ok(started)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_private_ops_only_when_enabled() {
        assert!(should_include_op("crypto.public.ws.trades.trade", false));
        assert!(!should_include_op(
            "crypto.private.ws.userdata.executionreport",
            false
        ));
        assert!(should_include_op(
            "crypto.private.ws.userdata.executionreport",
            true
        ));
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Build a generic JSON subscribe message for an exchange + op.
/// NOTE: This is a placeholder format used by the supervisor skeleton.
/// Each exchange crate's ws_manager.rs provides the authoritative exchange-specific
/// subscribe payload (e.g. Binance uses `{"method":"SUBSCRIBE","params":["<stream>"],"id":1}`).
fn build_subscribe_msg(exchange_id: &str, op_id: &str, symbol: Option<&str>) -> String {
    serde_json::json!({
        "method": "SUBSCRIBE",
        "params": [format!("{}:{}", op_id, symbol.unwrap_or("*"))],
        "exchange": exchange_id,
        "id": 1
    })
    .to_string()
}

/// Journal-first receive loop for a single WS connection.
/// Every inbound frame is written to the WAL before any further processing.
/// Returns when the connection drops or a stall timeout fires.
async fn run_receive_loop(
    conn_id: &str,
    exchange_id: &str,
    op_id: &str,
    symbol: Option<&str>,
    recv_rx: &mut tokio::sync::mpsc::Receiver<Vec<u8>>,
    wal: &mut WalWriter,
    metrics: &Arc<crate::metrics::IngestMetrics>,
    stall_timeout_secs: u64,
) -> Result<(), String> {
    use tokio::time::timeout;

    loop {
        match timeout(Duration::from_secs(stall_timeout_secs), recv_rx.recv()).await {
            Ok(Some(raw_bytes)) => {
                // JOURNAL-FIRST INVARIANT: persist raw bytes before any decode.
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let len = raw_bytes.len() as u64;
                let record = RawRecord {
                    ts,
                    exchange_id: exchange_id.to_string(),
                    conn_id: conn_id.to_string(),
                    op_id: op_id.to_string(),
                    symbol: symbol.map(ToOwned::to_owned),
                    raw_bytes_b64: base64::engine::general_purpose::STANDARD.encode(&raw_bytes),
                    meta: serde_json::json!({}),
                };
                wal.append(&record)?;
                metrics.record_journal_append(len);
            }
            Ok(None) => {
                return Err("connection_dropped".into());
            }
            Err(_elapsed) => {
                metrics.record_stall();
                return Err("stall_timeout".into());
            }
        }
    }
}

/// Spawn a background task that runs the full ingest loop for one exchange+op.
/// The loop connects, subscribes, journals every inbound frame, and reconnects
/// on disconnect using exponential back-off with jitter.
pub fn spawn_exchange_ingest(
    exchange_id: String,
    ws_op_id: String,
    symbol: Option<String>,
    cfg: Arc<IngestConfig>,
    metrics: Arc<crate::metrics::IngestMetrics>,
) {
    tokio::spawn(async move {
        let conn_id = format!("{exchange_id}-conn-1");
        // NOTE: conn-1 is used for the v1 single-connection skeleton.
        // Multi-connection support (as produced by the planner's ConnPlan) would require
        // one spawned task per ConnPlan entry; that wiring is left for the per-exchange
        // ws_manager.rs implementations.
        let key = subscription_key(&exchange_id, &ws_op_id, symbol.as_deref());
        let rules = load_for_exchange(std::path::Path::new(&cfg.rules_dir), &exchange_id);
        let stall_timeout = rules
            .heartbeat
            .as_ref()
            .and_then(|h| h.idle_timeout_secs)
            .unwrap_or(60);

        let mut wal = match WalWriter::open(&cfg.journal_dir, 64 * 1024 * 1024, FsyncMode::Balanced)
        {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Failed to open WAL for {exchange_id}: {e}");
                return;
            }
        };

        let mut attempt: u32 = 0;
        // NOTE: The WS URL below is a placeholder used in the connection supervisor skeleton.
        // Each exchange crate's ws_manager.rs is the authoritative source for its real WS URL
        // (loaded from the exchange's catalog.json).  Until per-exchange ws_manager
        // implementations are wired in, connection attempts will fail and trigger back-off.
        let ws_url = format!("wss://stream.{exchange_id}.com/ws");

        loop {
            match ws_open(&ws_url, 64, 1024).await {
                Err(e) => {
                    tracing::warn!("WS connect failed for {exchange_id}: {e}");
                    metrics.record_reconnect();
                }
                Ok(mut handle) => {
                    metrics.record_connected();
                    attempt = 0;

                    // Store operations are synchronous; drop the store before any `.await`
                    // so we don't hold the non-Send rusqlite Connection across await points.
                    let subscribe_msg = {
                        match SubscriptionStore::open(&cfg.store_path) {
                            Err(e) => {
                                tracing::error!("Store open failed: {e}");
                                return;
                            }
                            Ok(store) => {
                                let msg = if store.mark_inflight(&key, now_secs()).is_ok() {
                                    metrics.record_subscribe_sent();
                                    Some(build_subscribe_msg(
                                        &exchange_id,
                                        &ws_op_id,
                                        symbol.as_deref(),
                                    ))
                                } else {
                                    None
                                };
                                // store is dropped here, before any await
                                msg
                            }
                        }
                    };

                    if let Some(msg) = subscribe_msg {
                        if handle.send_tx.send(msg).await.is_err() {
                            metrics.record_subscribe_fail();
                            tracing::warn!(
                                exchange = %exchange_id,
                                op = %ws_op_id,
                                "subscribe send failed; channel closed"
                            );
                        }
                    }

                    // Receive loop: journal-first, no store reference held.
                    match run_receive_loop(
                        &conn_id,
                        &exchange_id,
                        &ws_op_id,
                        symbol.as_deref(),
                        &mut handle.recv_rx,
                        &mut wal,
                        &metrics,
                        stall_timeout,
                    )
                    .await
                    {
                        Ok(()) => {}
                        Err(reason) => {
                            tracing::warn!("{exchange_id} receive loop ended: {reason}");
                        }
                    }

                    // Requeue active subscriptions synchronously (store dropped immediately).
                    if let Ok(store) = SubscriptionStore::open(&cfg.store_path) {
                        if let Err(e) =
                            store.requeue_active_to_pending(&exchange_id, &conn_id, now_secs())
                        {
                            tracing::warn!(
                                exchange = %exchange_id,
                                conn = %conn_id,
                                "requeue_active_to_pending failed: {e}"
                            );
                        }
                    }
                    metrics.record_reconnect();
                }
            }

            let delay = backoff_with_jitter_ms(attempt, 500, 30_000, 1_000);
            tokio::time::sleep(Duration::from_millis(delay)).await;
            attempt = attempt.saturating_add(1);
        }
    });
}
