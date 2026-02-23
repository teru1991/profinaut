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

enum OutboundFrame {
    Text(String),
    Pong(Vec<u8>),
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
    let client_mps = 2u32; // conservative

    let mut sub_limiter = FixedWindowLimiter::new(subscribe_mps);
    let mut client_limiter = FixedWindowLimiter::new(client_mps);

    let hb_idle = rules
        .heartbeat
        .as_ref()
        .and_then(|h| h.idle_timeout_secs)
        .map(Duration::from_secs)
        .unwrap_or(cfg.idle_timeout);
    let ping_interval = rules
        .heartbeat
        .as_ref()
        .and_then(|h| h.ping_interval_secs)
        .unwrap_or(0);
    let ping_interval = if ping_interval == 0 {
        None
    } else {
        Some(Duration::from_secs(ping_interval))
    };

    let max_age = rules
        .heartbeat
        .as_ref()
        .and_then(|h| h.max_connection_age_secs)
        .unwrap_or(0);
    let max_age = if max_age == 0 {
        None
    } else {
        Some(Duration::from_secs(max_age))
    };

    let wal_policy = WalQueuePolicy::default();
    let send_bp = SendBackpressurePolicy::default();

    let mut reconnect_attempt: u32 = 0;
    let mut reconnect_times: VecDeque<Instant> = VecDeque::new();

    loop {
        if shutdown.is_triggered() {
            let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
            return Ok(());
        }

        let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());

        // storm guard
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
        let conn_started = Instant::now();

        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<OutboundFrame>(cfg.recv_queue_cap);
        let (wal_tx, mut wal_rx) = mpsc::channel::<RawRecord>(wal_policy.cap);

        let wal_writer = {
            let wal = wal.clone();
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                while let Some(rec) = wal_rx.recv().await {
                    if shutdown.is_triggered() {
                        break;
                    }
                    let r = {
                        let mut w = wal.lock().await;
                        w.append(&rec)
                    };
                    if let Err(e) = r {
                        warn!(exchange_id=%exchange_id, conn=%conn_id, err=%e, "WAL append failed");
                        break;
                    }
                }
            })
        };

        let writer = {
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown = shutdown.clone();
            let mut client_limiter = client_limiter.clone();
            tokio::spawn(async move {
                while let Some(f) = rx.recv().await {
                    if shutdown.is_triggered() {
                        break;
                    }
                    let w = client_limiter.allow_or_wait();
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }
                    let res = match f {
                        OutboundFrame::Text(t) => write.send(Message::Text(t)).await,
                        OutboundFrame::Pong(p) => write.send(Message::Pong(p)).await,
                    };
                    if res.is_err() {
                        warn!(exchange_id=%exchange_id, conn=%conn_id, "write failed");
                        break;
                    }
                }
            })
        };

        let mut last_inbound = Instant::now();
        let mut last_drip = Instant::now();
        let mut last_periodic_ping = Instant::now();

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

            if let Some(ma) = max_age {
                if conn_started.elapsed() >= ma {
                    warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "max_connection_age reached -> rotate reconnect");
                    break;
                }
            }

            if let Some(intv) = ping_interval {
                if last_periodic_ping.elapsed() >= intv {
                    last_periodic_ping = Instant::now();
                    if let Some(p) = adapter.ping_msg() {
                        let _ = tx.send(OutboundFrame::Text(p.text)).await;
                    }
                }
            }

            if last_inbound.elapsed() >= hb_idle {
                if let Some(p) = adapter.ping_msg() {
                    let _ = tx.send(OutboundFrame::Text(p.text)).await;
                }
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "idle timeout -> reconnect");
                break;
            }

            if let Some(until) = drip_paused_until {
                if Instant::now() >= until {
                    drip_paused_until = None;
                }
            }

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
                    let (_ex, op_id, sym_opt, params_canon) = match parse_stable_key(&key) {
                        Some(v) => v,
                        None => {
                            store.mark_deadletter(&key, "bad_key_format", now)?;
                            continue;
                        }
                    };
                    let symbol: &str = sym_opt.unwrap_or("");

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
                        let t0 = Instant::now();
                        if tx.send(OutboundFrame::Text(m.text)).await.is_err() {
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
                            writer.abort();
                            wal_writer.abort();
                            return Err("send backpressure stop".into());
                        }
                    }
                }
            }

            // inbound
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

            match msg {
                Message::Ping(p) => {
                    let _ = tx.send(OutboundFrame::Pong(p)).await;
                    last_inbound = Instant::now();
                    continue;
                }
                Message::Pong(_) => {
                    last_inbound = Instant::now();
                    continue;
                }
                Message::Close(_) => break,
                Message::Text(t) => {
                    handle_inbound(&adapter, &cfg, store, &wal_tx, &tx, t.into_bytes()).await?;
                    last_inbound = Instant::now();
                }
                Message::Binary(b) => {
                    handle_inbound(&adapter, &cfg, store, &wal_tx, &tx, b).await?;
                    last_inbound = Instant::now();
                }
                _ => {}
            }
        }

        writer.abort();
        wal_writer.abort();

        let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
        reconnect_attempt = reconnect_attempt.saturating_add(1);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
    }
}

async fn handle_inbound(
    adapter: &Arc<dyn WsVenueAdapter>,
    cfg: &WsRunConfig,
    store: &mut SubscriptionStore,
    wal_tx: &mpsc::Sender<RawRecord>,
    out_tx: &mpsc::Sender<OutboundFrame>,
    raw: Vec<u8>,
) -> Result<(), String> {
    if raw.len() > cfg.max_frame_bytes {
        return Err("frame too large (DoS) -> stop".into());
    }

    let classified = adapter.classify_inbound(&raw);

    if let InboundClass::Respond { msg } = &classified {
        let _ = out_tx.send(OutboundFrame::Text(msg.text.clone())).await;
    }

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
        InboundClass::Respond { .. } => {
            meta.insert("kind".into(), serde_json::Value::String("respond".into()));
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

    // WAL queue bounded: await send (safe)
    wal_tx
        .send(rec)
        .await
        .map_err(|_| "WAL writer stopped -> stop".to_string())?;

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

    Ok(())
}
