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
    let symbol = if parts[2].is_empty() {
        None
    } else {
        Some(parts[2])
    };
    let params = parts[3];
    Some((exchange, op, symbol, params))
}

/// 固定窓 msgs/sec リミッタ（待つ）
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

/// Phase2: WAL writer task parameters
#[derive(Clone, Debug)]
struct WalQueuePolicy {
    cap: usize,
    stop_on_full: bool,
}
impl Default for WalQueuePolicy {
    fn default() -> Self {
        Self {
            cap: 8192,
            stop_on_full: true,
        }
    }
}

/// Phase2: send backpressure policy (send enqueue wait time)
#[derive(Clone, Debug)]
struct SendBackpressurePolicy {
    throttle_after_ms: u64,
    throttle_consecutive: u32,
    stop_after_ms: u64,
    stop_consecutive: u32,
    throttle_pause: Duration,
}
impl Default for SendBackpressurePolicy {
    fn default() -> Self {
        Self {
            throttle_after_ms: 200,
            throttle_consecutive: 5,
            stop_after_ms: 1000,
            stop_consecutive: 10,
            throttle_pause: Duration::from_secs(2),
        }
    }
}

/// WS connection runner (public)
pub async fn run_ws_connection(
    adapter: Arc<dyn WsVenueAdapter>,
    rules: ExchangeWsRules,
    store: &mut SubscriptionStore,
    wal: Arc<Mutex<ucel_journal::WalWriter>>,
    cfg: WsRunConfig,
    shutdown: ShutdownToken,
) -> Result<(), String> {
    let url = adapter.ws_url();

    // subscribe rate (rules). if missing -> safe 1 msg/sec
    let subscribe_mps = rules
        .rate
        .as_ref()
        .and_then(|r| r.messages_per_second)
        .unwrap_or(1)
        .max(1);

    // client msg rate (ping含む). conservative
    let client_mps = 2u32;

    let mut sub_limiter = FixedWindowLimiter::new(subscribe_mps);
    let mut client_limiter = FixedWindowLimiter::new(client_mps);

    let hb_idle = rules
        .heartbeat
        .as_ref()
        .and_then(|h| h.idle_timeout_secs)
        .map(Duration::from_secs)
        .unwrap_or(cfg.idle_timeout);

    let wal_policy = WalQueuePolicy::default();
    let send_bp = SendBackpressurePolicy::default();

    let mut reconnect_attempt: u32 = 0;
    let mut reconnect_times: VecDeque<Instant> = VecDeque::new();

    loop {
        if shutdown.is_triggered() {
            let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
            return Ok(());
        }

        // requeue (続きから)
        let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());

        // storm guard window management
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
            return Err(format!(
                "reconnect storm detected: {}",
                reconnect_times.len()
            ));
        }

        info!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, url=%url, "ws connecting");

        let (ws_stream, _) = match tokio::time::timeout(cfg.connect_timeout, connect_async(&url))
            .await
        {
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

        // outbound queue
        let (tx, mut rx) = mpsc::channel::<OutboundMsg>(cfg.recv_queue_cap);

        // WAL queue: read loop -> wal_tx (bounded)
        let (wal_tx, mut wal_rx) = mpsc::channel::<RawRecord>(wal_policy.cap);

        // WAL writer task (append-first guarantee, but separated from read loop)
        let wal_writer = {
            let wal = wal.clone();
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                while let Some(rec) = wal_rx.recv().await {
                    if shutdown.is_triggered() {
                        // best-effort drain could be added; v1: stop quickly
                        break;
                    }
                    let t0 = Instant::now();
                    let r = {
                        let mut w = wal.lock().await;
                        w.append(&rec)
                    };
                    if let Err(e) = r {
                        // WAL failed => safety stop at upper layer by closing channel (no retries here)
                        warn!(exchange_id=%exchange_id, conn=%conn_id, err=%e, "WAL append failed");
                        break;
                    }
                    let dt = t0.elapsed();
                    if dt > Duration::from_millis(500) {
                        warn!(exchange_id=%exchange_id, conn=%conn_id, ms=?dt.as_millis(), "WAL append slow");
                    }
                }
            })
        };

        // writer task (ALL outbound must respect client limiter)
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
                    if write.send(Message::Text(m.text)).await.is_err() {
                        warn!(exchange_id=%exchange_id, conn=%conn_id, "write failed");
                        break;
                    }
                }
            })
        };

        let mut last_inbound = Instant::now();
        let mut last_drip = Instant::now();

        // backpressure state
        let mut drip_paused_until: Option<Instant> = None;
        let mut slow_send_consecutive: u32 = 0;
        let mut very_slow_send_consecutive: u32 = 0;

        loop {
            if shutdown.is_triggered() {
                let _ =
                    store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
                writer.abort();
                wal_writer.abort();
                return Ok(());
            }

            // idle -> ping/reconnect
            if last_inbound.elapsed() >= hb_idle {
                if let Some(p) = adapter.ping_msg() {
                    let _ = tx.send(p).await;
                }
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "idle timeout -> reconnect");
                break;
            }

            // drip pause due to backpressure
            if let Some(until) = drip_paused_until {
                if Instant::now() < until {
                    // still paused: no drip, just read
                } else {
                    drip_paused_until = None;
                }
            }

            // drip subscribe (unless paused)
            if drip_paused_until.is_none() && last_drip.elapsed() >= Duration::from_millis(200) {
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

                    // subscribe rate
                    let w = sub_limiter.allow_or_wait();
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }

                    let params: Value = serde_json::from_str(params_canon)
                        .unwrap_or_else(|_| serde_json::json!({}));
                    let msgs = match adapter.build_subscribe(op_id, symbol, &params) {
                        Ok(m) => m,
                        Err(e) => {
                            store.mark_deadletter(&key, &format!("build_subscribe:{e}"), now)?;
                            continue;
                        }
                    };

                    for m in msgs {
                        // Phase2: send enqueue latency as backpressure signal
                        let t0 = Instant::now();
                        if tx.send(m).await.is_err() {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "outbound queue closed");
                            break;
                        }
                        let dt = t0.elapsed();

                        if dt >= Duration::from_millis(send_bp.stop_after_ms) {
                            very_slow_send_consecutive += 1;
                        } else {
                            very_slow_send_consecutive = 0;
                        }

                        if dt >= Duration::from_millis(send_bp.throttle_after_ms) {
                            slow_send_consecutive += 1;
                        } else {
                            slow_send_consecutive = 0;
                        }

                        if slow_send_consecutive >= send_bp.throttle_consecutive {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "send backpressure -> throttle drip");
                            drip_paused_until = Some(Instant::now() + send_bp.throttle_pause);
                            slow_send_consecutive = 0;
                        }

                        if very_slow_send_consecutive >= send_bp.stop_consecutive {
                            // Safety stop (cannot keep up)
                            writer.abort();
                            wal_writer.abort();
                            return Err("send backpressure stop".into());
                        }
                    }
                }
            }

            // inbound read (short timeout)
            let next = tokio::time::timeout(Duration::from_millis(250), read.next()).await;
            let maybe_item = match next {
                Ok(v) => v,
                Err(_) => None,
            };
            let Some(item) = maybe_item else {
                continue;
            };

            let msg = match item {
                Ok(m) => m,
                Err(_) => break,
            };

            let raw: Vec<u8> = match msg {
                Message::Text(t) => t.into_bytes(),
                Message::Binary(b) => b,
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Close(_) => break,
                _ => continue,
            };

            if raw.len() > cfg.max_frame_bytes {
                writer.abort();
                wal_writer.abort();
                return Err("frame too large (DoS) -> stop".into());
            }

            last_inbound = Instant::now();

            // classify (light)
            let classified = adapter.classify_inbound(&raw);

            // Phase2: enqueue WAL record (append-first intent) via bounded queue.
            // If queue is full -> Stop (safe)
            let mut meta = serde_json::Map::new();
            match &classified {
                InboundClass::Ack { op_id, symbol, .. } => {
                    meta.insert("op_id".into(), serde_json::Value::String(op_id.clone()));
                    if let Some(s) = symbol.as_ref() {
                        meta.insert("symbol".into(), serde_json::Value::String(s.clone()));
                    }
                    meta.insert("kind".into(), serde_json::Value::String("ack".into()));
                }
                InboundClass::Data { op_id, symbol, .. } => {
                    if let Some(op) = op_id.as_ref() {
                        meta.insert("op_id".into(), serde_json::Value::String(op.clone()));
                    }
                    if let Some(s) = symbol.as_ref() {
                        meta.insert("symbol".into(), serde_json::Value::String(s.clone()));
                    }
                    meta.insert("kind".into(), serde_json::Value::String("data".into()));
                }
                InboundClass::Nack {
                    reason,
                    op_id,
                    symbol,
                    ..
                } => {
                    meta.insert("reason".into(), serde_json::Value::String(reason.clone()));
                    if let Some(op) = op_id.as_ref() {
                        meta.insert("op_id".into(), serde_json::Value::String(op.clone()));
                    }
                    if let Some(s) = symbol.as_ref() {
                        meta.insert("symbol".into(), serde_json::Value::String(s.clone()));
                    }
                    meta.insert("kind".into(), serde_json::Value::String("nack".into()));
                }
                _ => {}
            }

            let rec = RawRecord {
                ts: now_unix_u64(),
                exchange_id: cfg.exchange_id.clone(),
                conn_id: cfg.conn_id.clone(),
                op_id: meta
                    .get("op_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                symbol: meta
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                raw_bytes_b64: base64::encode(&raw),
                meta: serde_json::Value::Object(meta),
            };

            // bounded WAL queue: full => stop
            if wal_policy.stop_on_full {
                match wal_tx.try_send(rec) {
                    Ok(_) => {}
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        writer.abort();
                        wal_writer.abort();
                        return Err("WAL queue full -> stop".into());
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        writer.abort();
                        wal_writer.abort();
                        return Err("WAL writer stopped -> stop".into());
                    }
                }
            } else if wal_tx.send(rec).await.is_err() {
                writer.abort();
                wal_writer.abort();
                return Err("WAL writer stopped -> stop".into());
            }

            // Active marking (store)
            let now = now_unix_i64();
            match classified {
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

        // disconnect -> reconnect
        writer.abort();
        wal_writer.abort();

        let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
        reconnect_attempt = reconnect_attempt.saturating_add(1);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
    }
}
