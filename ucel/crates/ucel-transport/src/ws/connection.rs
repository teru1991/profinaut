use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn};

use ucel_journal::RawRecord;
use ucel_subscription_store::SubscriptionStore;
use ucel_ws_rules::ExchangeWsRules;

use super::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};
use super::reconnect::{backoff_with_jitter_ms, storm_guard};

#[derive(Clone, Debug)]
pub struct WsRunConfig {
    pub exchange_id: String,
    pub conn_id: String,
    pub recv_queue_cap: usize,
    pub max_frame_bytes: usize,
    pub max_inflight_per_conn: usize,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub reconnect_storm_window: Duration,
    pub reconnect_storm_max: usize,
}

#[derive(Clone)]
pub struct ShutdownToken {
    pub flag: Arc<std::sync::atomic::AtomicBool>,
}
impl ShutdownToken {
    pub fn is_triggered(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.flag.load(Ordering::SeqCst)
    }
}

fn now_unix_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn now_unix_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn parse_stable_key(key: &str) -> Option<(&str, &str, Option<&str>, &str)> {
    let parts: Vec<&str> = key.split('|').collect();
    if parts.len() != 4 {
        return None;
    }
    let exchange = parts[0];
    let op = parts[1];
    let symbol = if parts[2].is_empty() { None } else { Some(parts[2]) };
    let params = parts[3];
    Some((exchange, op, symbol, params))
}

#[derive(Clone)]
struct FixedWindowLimiter {
    max_per_sec: u32,
    window_start: Instant,
    used: u32,
}
impl FixedWindowLimiter {
    fn new(max_per_sec: u32) -> Self {
        Self {
            max_per_sec: max_per_sec.max(1),
            window_start: Instant::now(),
            used: 0,
        }
    }
    fn allow_or_wait(&mut self) -> Duration {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.used = 0;
        }
        if self.used < self.max_per_sec {
            self.used += 1;
            Duration::from_secs(0)
        } else {
            Duration::from_secs(1).saturating_sub(now.duration_since(self.window_start))
        }
    }
}

pub async fn run_ws_connection(
    adapter: Arc<dyn WsVenueAdapter>,
    rules: ExchangeWsRules,
    store: &mut SubscriptionStore,
    wal: Arc<Mutex<ucel_journal::WalWriter>>,
    cfg: WsRunConfig,
    shutdown: ShutdownToken,
) -> Result<(), String> {
    let url = adapter.ws_url();

    let subscribe_mps = rules
        .rate
        .as_ref()
        .and_then(|r| r.messages_per_second)
        .unwrap_or(1)
        .max(1);

    let client_mps = 2u32;

    let mut sub_limiter = FixedWindowLimiter::new(subscribe_mps);
    let client_limiter = FixedWindowLimiter::new(client_mps);

    let hb_idle = rules
        .heartbeat
        .as_ref()
        .and_then(|h| h.idle_timeout_secs)
        .map(Duration::from_secs)
        .unwrap_or(cfg.idle_timeout);

    let mut reconnect_attempt: u32 = 0;
    let mut reconnect_times: VecDeque<Instant> = VecDeque::new();

    loop {
        if shutdown.is_triggered() {
            let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
            return Ok(());
        }

        let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());

        let nowi = Instant::now();
        reconnect_times.push_back(nowi);
        while let Some(front) = reconnect_times.front() {
            if nowi.duration_since(*front) > cfg.reconnect_storm_window {
                reconnect_times.pop_front();
            } else {
                break;
            }
        }
        if !storm_guard(reconnect_times.len(), cfg.reconnect_storm_max) {
            return Err(format!("reconnect storm detected: {}", reconnect_times.len()));
        }

        info!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, url=%url, "ws connecting");

        let (ws_stream, _) = match tokio::time::timeout(cfg.connect_timeout, connect_async(&url)).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, err=%e, backoff_ms=backoff, "connect error");
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                continue;
            }
            Err(_) => {
                let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, backoff_ms=backoff, "connect timeout");
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                continue;
            }
        };

        reconnect_attempt = 0;

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<OutboundMsg>(cfg.recv_queue_cap);

        let writer = {
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown = shutdown.clone();
            let mut client_limiter = client_limiter.clone();
            tokio::spawn(async move {
                while let Some(m) = rx.recv().await {
                    if shutdown.is_triggered() {
                        break;
                    }
                    let w = client_limiter.allow_or_wait();
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }
                    if write.send(Message::Text(m.text.into())).await.is_err() {
                        warn!(exchange_id=%exchange_id, conn=%conn_id, "write failed");
                        break;
                    }
                }
            })
        };

        let mut last_inbound = Instant::now();
        let mut last_drip = Instant::now();

        loop {
            if shutdown.is_triggered() {
                let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
                writer.abort();
                return Ok(());
            }

            if last_drip.elapsed() >= Duration::from_millis(200) {
                last_drip = Instant::now();
                let now = now_unix_i64();
                let keys = store.next_pending_batch(
                    &cfg.exchange_id,
                    &cfg.conn_id,
                    cfg.max_inflight_per_conn,
                    now,
                )?;

                for key in keys {
                    if shutdown.is_triggered() {
                        break;
                    }

                    let (_ex, op_id, sym_opt, params_canon) = match parse_stable_key(&key) {
                        Some(v) => v,
                        None => {
                            store.mark_deadletter(&key, "bad_key_format", now)?;
                            continue;
                        }
                    };
                    let symbol = match sym_opt {
                        Some(s) => s,
                        None => {
                            store.mark_deadletter(&key, "symbol_missing", now)?;
                            continue;
                        }
                    };

                    let w = sub_limiter.allow_or_wait();
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }

                    let params: Value =
                        serde_json::from_str(params_canon).unwrap_or_else(|_| serde_json::json!({}));
                    match adapter.build_subscribe(op_id, symbol, &params) {
                        Ok(msgs) => {
                            for m in msgs {
                                let _ = tx.send(m).await;
                            }
                        }
                        Err(e) => {
                            store.mark_deadletter(&key, &format!("build_subscribe:{e}"), now)?;
                        }
                    }
                }
            }

            if last_inbound.elapsed() >= hb_idle {
                if let Some(p) = adapter.ping_msg() {
                    let _ = tx.send(p).await;
                }
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "idle timeout -> reconnect");
                break;
            }

            let next = tokio::time::timeout(Duration::from_millis(250), read.next()).await;
            let Some(item) = (match next {
                Ok(v) => v,
                Err(_) => None,
            }) else {
                continue;
            };
            let msg = match item {
                Ok(m) => m,
                Err(_) => break,
            };

            let raw: Vec<u8> = match msg {
                Message::Text(t) => t.bytes().collect(),
                Message::Binary(b) => b.to_vec(),
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Close(_) => break,
                _ => continue,
            };

            if raw.len() > cfg.max_frame_bytes {
                writer.abort();
                return Err("frame too large (DoS) -> stop".into());
            }

            last_inbound = Instant::now();

            let rec = RawRecord {
                ts: now_unix_u64(),
                exchange_id: cfg.exchange_id.clone(),
                conn_id: cfg.conn_id.clone(),
                op_id: "unknown".to_string(),
                symbol: None,
                raw_bytes_b64: {
                    use base64::{engine::general_purpose::STANDARD, Engine as _};
                    STANDARD.encode(&raw)
                },
                meta: serde_json::json!({}),
            };
            {
                let mut w = wal.lock().await;
                w.append(&rec)?;
            }

            let now = now_unix_i64();
            match adapter.classify_inbound(&raw) {
                InboundClass::Ack {
                    op_id,
                    symbol,
                    params_canon_hint,
                } => {
                    let params_canon = params_canon_hint.as_deref().unwrap_or("{}");
                    let key = store.find_key_by_fields(
                        &cfg.exchange_id,
                        &cfg.conn_id,
                        &op_id,
                        symbol.as_deref(),
                        params_canon,
                    )?;
                    if let Some(k) = key {
                        store.mark_active(&k, now)?;
                        store.bump_last_message(&k, now)?;
                    }
                }
                InboundClass::Data {
                    op_id,
                    symbol,
                    params_canon_hint,
                } => {
                    if let (Some(op), Some(sym)) = (op_id, symbol) {
                        let params_canon = params_canon_hint.as_deref().unwrap_or("{}");
                        let key = store.find_key_by_fields(
                            &cfg.exchange_id,
                            &cfg.conn_id,
                            &op,
                            Some(&sym),
                            params_canon,
                        )?;
                        if let Some(k) = key {
                            store.mark_active(&k, now)?;
                            store.bump_last_message(&k, now)?;
                        }
                    }
                }
                InboundClass::Nack {
                    reason,
                    op_id,
                    symbol,
                    params_canon_hint,
                } => {
                    let params_canon = params_canon_hint.as_deref().unwrap_or("{}");
                    if let Some(op) = op_id {
                        let key = store.find_key_by_fields(
                            &cfg.exchange_id,
                            &cfg.conn_id,
                            &op,
                            symbol.as_deref(),
                            params_canon,
                        )?;
                        if let Some(k) = key {
                            store.mark_deadletter(&k, &format!("nack:{reason}"), now)?;
                        }
                    }
                }
                _ => {}
            }
        }

        writer.abort();

        let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
        reconnect_attempt = reconnect_attempt.saturating_add(1);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
    }
}
