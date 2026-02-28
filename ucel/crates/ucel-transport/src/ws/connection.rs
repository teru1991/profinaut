use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn};

use ucel_journal::RawRecord;

use crate::stability::events::{
    ConnState, ReconnectReason, ShutdownPhase, TransportStabilityEvent,
};
use crate::stability::{map_breaker_state, map_outcome, StabilityHub};
use ucel_subscription_store::SubscriptionStore;
use ucel_ws_rules::ExchangeWsRules;

use super::adapter::{InboundClass, WsVenueAdapter};
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitDecision};
use super::limiter::{BucketConfig, WsRateLimiter, WsRateLimiterConfig};
use super::overflow::{DropMode, OverflowPolicy, Spooler, SpoolerConfig};
use super::priority::{
    classify_op_id_priority, OutboundPriority, PriorityQueue, PushOutcome, QueuedOutbound,
    WsOutboundFrame,
};
use super::reconnect::{backoff_with_jitter_ms, storm_guard};
use super::shutdown::{graceful_shutdown_ws, GracefulShutdownConfig};

#[derive(Clone, Debug)]
pub struct WsRunConfig {
    pub exchange_id: String,
    pub conn_id: String,

    /// outbound queue capacity (writer queue)
    pub out_queue_cap: usize,

    /// WAL queue capacity
    pub wal_queue_cap: usize,

    /// inbound frame safety
    pub max_frame_bytes: usize,

    /// max inflight pulled from store per drip tick
    pub max_inflight_per_conn: usize,

    pub connect_timeout: Duration,
    pub idle_timeout: Duration,

    /// reconnect storm guard params
    pub reconnect_storm_window: Duration,
    pub reconnect_storm_max: usize,

    /// subscription stale detection (auto requeue -> auto resubscribe)
    pub stale_after: Duration,
    pub stale_sweep_interval: Duration,
    pub stale_max_batch: usize,

    /// circuit breaker
    pub breaker: CircuitBreakerConfig,

    /// overflow/backpressure policy
    pub overflow: WsOverflowConfig,

    /// graceful shutdown config
    pub graceful: GracefulShutdownConfig,
}

#[derive(Clone, Debug)]
pub enum WsOverflowMode {
    DropNewest,
    DropOldestLowPriority,
    SlowDownThenDropOldestLowPriority,
    SpillToDiskThenDropOldestLowPriority,
}

#[derive(Clone, Debug)]
pub struct WsOverflowConfig {
    pub mode: WsOverflowMode,
    /// used only for SpillToDisk*
    pub spill_dir: Option<String>,
    /// used only for SlowDown*
    pub slowdown_max_wait: Duration,
}

impl Default for WsOverflowConfig {
    fn default() -> Self {
        Self {
            mode: WsOverflowMode::SlowDownThenDropOldestLowPriority,
            spill_dir: None,
            slowdown_max_wait: Duration::from_millis(200),
        }
    }
}

impl Default for WsRunConfig {
    fn default() -> Self {
        Self {
            exchange_id: "unknown".into(),
            conn_id: "conn".into(),
            out_queue_cap: 256,
            wal_queue_cap: 8192,
            max_frame_bytes: 1024 * 1024,
            max_inflight_per_conn: 10,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(30),
            reconnect_storm_window: Duration::from_secs(30),
            reconnect_storm_max: 10,
            stale_after: Duration::from_secs(60),
            stale_sweep_interval: Duration::from_secs(5),
            stale_max_batch: 200,
            breaker: CircuitBreakerConfig::default(),
            overflow: WsOverflowConfig::default(),
            graceful: GracefulShutdownConfig::default(),
        }
    }
}

/// Backward-compat re-export (tests/importers may refer to ws::connection::ShutdownToken)
pub use super::shutdown::ShutdownToken;

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

fn build_overflow_policy(cfg: &WsRunConfig) -> Result<OverflowPolicy, String> {
    match cfg.overflow.mode {
        WsOverflowMode::DropNewest => Ok(OverflowPolicy::Drop {
            mode: DropMode::DropNewest,
        }),
        WsOverflowMode::DropOldestLowPriority => Ok(OverflowPolicy::Drop {
            mode: DropMode::DropOldestLowPriority,
        }),
        WsOverflowMode::SlowDownThenDropOldestLowPriority => Ok(OverflowPolicy::SlowDown {
            max_wait: cfg.overflow.slowdown_max_wait,
            fallback: DropMode::DropOldestLowPriority,
        }),
        WsOverflowMode::SpillToDiskThenDropOldestLowPriority => {
            let dir =
                cfg.overflow.spill_dir.as_ref().ok_or_else(|| {
                    "overflow.spill_dir is required for SpillToDisk mode".to_string()
                })?;
            let sp = Spooler::open(SpoolerConfig::new(dir))?;
            Ok(OverflowPolicy::SpillToDisk {
                spooler: Arc::new(sp),
                fallback: DropMode::DropOldestLowPriority,
            })
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

    // ===== Rate limiting (bucketed + private priority) =====
    //
    // rules.rate.messages_per_second を public の基準にし、private/control は安全側に別bucket。
    // 取引所ごとに toml で後から調整できるよう、ここは「デフォルト安全」。
    let public_rps = rules
        .rate
        .as_ref()
        .and_then(|r| r.messages_per_second)
        .unwrap_or(1)
        .max(1) as f64;

    // private は public より少し低くする（安全側）
    // 本番では取引所特性に合わせて rules 側へ拡張してもOK。
    let private_rps = (public_rps / 2.0).max(1.0);
    let control_rps = (public_rps * 2.0).max(2.0);

    let ws_limiter = Arc::new(Mutex::new(WsRateLimiter::new(WsRateLimiterConfig {
        control: BucketConfig::per_second(control_rps),
        private: BucketConfig::per_second(private_rps),
        public: BucketConfig::per_second(public_rps),
        min_gap: Duration::from_millis(0),
    })));

    // Heartbeat / idle
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

    let overflow_policy = build_overflow_policy(&cfg)?;
    let mut breaker = CircuitBreaker::new(cfg.breaker.clone());
    let stability = Arc::new(StabilityHub::new());

    let mut reconnect_attempt: u32 = 0;
    let mut reconnect_times: VecDeque<Instant> = VecDeque::new();

    // === Outer reconnect loop ===
    loop {
        if shutdown.is_triggered() {
            stability.emit(TransportStabilityEvent::ReconnectAttempt {
                exchange_id: cfg.exchange_id.clone(),
                conn_id: cfg.conn_id.clone(),
                reason: ReconnectReason::Shutdown,
                attempt: reconnect_attempt as u64,
            });
            let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());
            return Ok(());
        }

        // Always start by requeueing active/inflight to pending before a fresh connect.
        let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());

        // Storm guard
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

        // Circuit breaker (Open cooldown / HalfOpen trial)
        match breaker.before_attempt(Instant::now()) {
            CircuitDecision::Allow => {}
            CircuitDecision::Wait(d) => {
                warn!(
                    exchange_id=%cfg.exchange_id,
                    conn=%cfg.conn_id,
                    wait_ms=d.as_millis() as u64,
                    state=?breaker.kind(),
                    "circuit breaker open -> waiting"
                );
                stability.emit(TransportStabilityEvent::CircuitBreakerState {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    state: map_breaker_state(breaker.kind()),
                });
                stability.emit(TransportStabilityEvent::ReconnectAttempt {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    reason: ReconnectReason::CircuitOpenWait,
                    attempt: reconnect_attempt as u64,
                });
                tokio::time::sleep(d).await;
                continue;
            }
        }

        if breaker.kind() == super::circuit_breaker::CircuitStateKind::HalfOpen {
            breaker.on_half_open_trial();
        }

        info!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, url=%url, "ws connecting");

        let (ws_stream, _) = match tokio::time::timeout(cfg.connect_timeout, connect_async(&url))
            .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                breaker.on_failure(Instant::now());
                let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, err=%e, backoff_ms=backoff, "connect error");
                stability.emit(TransportStabilityEvent::ReconnectAttempt {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    reason: ReconnectReason::ConnectError,
                    attempt: reconnect_attempt as u64,
                });
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                continue;
            }
            Err(_) => {
                breaker.on_failure(Instant::now());
                let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, backoff_ms=backoff, "connect timeout");
                stability.emit(TransportStabilityEvent::ReconnectAttempt {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    reason: ReconnectReason::ConnectTimeout,
                    attempt: reconnect_attempt as u64,
                });
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                continue;
            }
        };

        // Connection established => mark breaker success (we got a socket)
        breaker.on_success(Instant::now());
        reconnect_attempt = 0;

        let conn_started = Instant::now();
        let (mut write, mut read) = ws_stream.split();
        stability.emit(TransportStabilityEvent::ConnectionState {
            exchange_id: cfg.exchange_id.clone(),
            conn_id: cfg.conn_id.clone(),
            state: ConnState::Connected,
        });
        stability.add_gauge("active_conn", 1);

        // Priority outbound queue (private-first) + graceful close marker.
        let outq = PriorityQueue::new(cfg.out_queue_cap);

        // WAL queue
        let (wal_tx, mut wal_rx) = mpsc::channel::<RawRecord>(cfg.wal_queue_cap);

        // WAL writer task (drain-friendly)
        let wal_writer = {
            let wal = wal.clone();
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown2 = shutdown.clone();
            let stability2 = stability.clone();
            tokio::spawn(async move {
                loop {
                    if shutdown2.is_triggered() && wal_rx.is_empty() {
                        break;
                    }
                    stability2.set_gauge("wal_queue_len", wal_rx.len() as i64);
                    let next =
                        tokio::time::timeout(Duration::from_millis(200), wal_rx.recv()).await;
                    let Some(rec) = next.ok().flatten() else {
                        continue;
                    };

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

        // Writer task (drain-friendly, close->flush->join works)
        let writer = {
            let exchange_id = cfg.exchange_id.clone();
            let conn_id = cfg.conn_id.clone();
            let shutdown2 = shutdown.clone();
            let outq2 = outq.clone();
            let ws_limiter2 = ws_limiter.clone();
            let stability2 = stability.clone();
            tokio::spawn(async move {
                loop {
                    if shutdown2.is_triggered() && outq2.is_empty().await {
                        break;
                    }
                    stability2.set_gauge("outq_len", outq2.len() as i64);
                    let Some(item) = outq2.recv().await else {
                        break;
                    };

                    // Rate limit by priority bucket
                    let w = {
                        let mut lim = ws_limiter2.lock().await;
                        lim.acquire_wait(item.priority, Instant::now())
                    };
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }

                    match item.frame {
                        WsOutboundFrame::CloseRequest => {
                            // Send close frame and exit.
                            let _ = write.send(Message::Close(None)).await;
                            break;
                        }
                        WsOutboundFrame::Text(t) => {
                            if write.send(Message::Text(t)).await.is_err() {
                                warn!(exchange_id=%exchange_id, conn=%conn_id, "write failed");
                                break;
                            }
                        }
                        WsOutboundFrame::Pong(p) => {
                            if write.send(Message::Pong(p)).await.is_err() {
                                warn!(exchange_id=%exchange_id, conn=%conn_id, "write failed");
                                break;
                            }
                        }
                    }
                }
            })
        };

        // Timers
        let mut last_inbound = Instant::now();
        let mut last_drip = Instant::now();
        let mut last_periodic_ping = Instant::now();
        let mut last_stale_sweep = Instant::now();

        // === Inner run loop ===
        loop {
            // Shutdown => graceful path (close->flush->requeue->join)
            if shutdown.is_triggered() {
                stability.emit(TransportStabilityEvent::ConnectionState {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    state: ConnState::ShuttingDown,
                });
                stability.emit(TransportStabilityEvent::ShutdownPhase {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    phase: ShutdownPhase::CloseRequested,
                });
                let _ = graceful_shutdown_ws(
                    cfg.graceful.clone(),
                    &cfg.exchange_id,
                    &cfg.conn_id,
                    store,
                    &outq,
                    &shutdown,
                    writer,
                    wal_writer,
                )
                .await;
                stability.emit(TransportStabilityEvent::ShutdownPhase {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    phase: ShutdownPhase::Joined,
                });
                stability.emit(TransportStabilityEvent::ConnectionState {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    state: ConnState::Disconnected,
                });
                stability.add_gauge("active_conn", -1);
                return Ok(());
            }

            // Preemptive rotate
            if let Some(ma) = max_age {
                if conn_started.elapsed() >= ma {
                    warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "max_connection_age reached -> rotate reconnect");
                    stability.emit(TransportStabilityEvent::ReconnectAttempt {
                        exchange_id: cfg.exchange_id.clone(),
                        conn_id: cfg.conn_id.clone(),
                        reason: ReconnectReason::MaxAge,
                        attempt: reconnect_attempt as u64,
                    });
                    break;
                }
            }

            // Periodic ping (venue adapter)
            if let Some(intv) = ping_interval {
                if last_periodic_ping.elapsed() >= intv {
                    last_periodic_ping = Instant::now();
                    if let Some(p) = adapter.ping_msg() {
                        let out = outq
                            .push(
                                &cfg.exchange_id,
                                &cfg.conn_id,
                                QueuedOutbound {
                                    priority: OutboundPriority::Control,
                                    op_id: Some("ws.control.ping".into()),
                                    symbol: None,
                                    frame: WsOutboundFrame::Text(p.text),
                                    meta: serde_json::json!({"kind":"ping"}),
                                },
                                &overflow_policy,
                                now_unix_u64(),
                            )
                            .await
                            .unwrap_or(PushOutcome::Dropped);
                        stability.emit(TransportStabilityEvent::OutqOverflowOutcome {
                            exchange_id: cfg.exchange_id.clone(),
                            conn_id: cfg.conn_id.clone(),
                            outcome: map_outcome(out),
                        });
                        stability.emit(TransportStabilityEvent::OutqOverflowOutcome {
                            exchange_id: cfg.exchange_id.clone(),
                            conn_id: cfg.conn_id.clone(),
                            outcome: map_outcome(out),
                        });
                        match out {
                            PushOutcome::Enqueued => {}
                            PushOutcome::Dropped => {
                                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.ping", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> dropped");
                            }
                            PushOutcome::Spilled => {
                                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.ping", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> spilled-to-disk");
                            }
                        }
                    }
                }
            }

            // Idle detection => reconnect
            if last_inbound.elapsed() >= hb_idle {
                // Try ping once before breaking (best effort)
                if let Some(p) = adapter.ping_msg() {
                    let out = outq
                        .push(
                            &cfg.exchange_id,
                            &cfg.conn_id,
                            QueuedOutbound {
                                priority: OutboundPriority::Control,
                                op_id: Some("ws.control.ping".into()),
                                symbol: None,
                                frame: WsOutboundFrame::Text(p.text),
                                meta: serde_json::json!({"kind":"ping"}),
                            },
                            &overflow_policy,
                            now_unix_u64(),
                        )
                        .await
                        .unwrap_or(PushOutcome::Dropped);
                    match out {
                        PushOutcome::Enqueued => {}
                        PushOutcome::Dropped => {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.ping", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> dropped");
                        }
                        PushOutcome::Spilled => {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.ping", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> spilled-to-disk");
                        }
                    }
                }
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, "idle timeout -> reconnect");
                stability.emit(TransportStabilityEvent::ReconnectAttempt {
                    exchange_id: cfg.exchange_id.clone(),
                    conn_id: cfg.conn_id.clone(),
                    reason: ReconnectReason::IdleTimeout,
                    attempt: reconnect_attempt as u64,
                });
                break;
            }

            // Stale sweep => active stale -> pending (auto resubscribe)
            if last_stale_sweep.elapsed() >= cfg.stale_sweep_interval {
                last_stale_sweep = Instant::now();
                let now = now_unix_i64();
                let stale_after_secs = cfg.stale_after.as_secs() as i64;
                let changed = store
                    .requeue_stale_active_to_pending(
                        &cfg.exchange_id,
                        &cfg.conn_id,
                        stale_after_secs,
                        cfg.stale_max_batch,
                        now,
                    )
                    .unwrap_or(0);

                if changed > 0 {
                    warn!(
                        exchange_id=%cfg.exchange_id,
                        conn=%cfg.conn_id,
                        changed,
                        "stale subscriptions requeued -> will resubscribe"
                    );
                }
            }

            // Drip pending subscriptions (private priority via op_id classifier)
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
                    let (_ex, op_id, sym_opt, params_canon) = match parse_stable_key(&key) {
                        Some(v) => v,
                        None => {
                            store.mark_deadletter(&key, "bad_key_format", now)?;
                            continue;
                        }
                    };

                    let symbol: &str = sym_opt.unwrap_or("");

                    let params: Value = serde_json::from_str(params_canon)
                        .unwrap_or_else(|_| serde_json::json!({}));

                    let msgs = match adapter.build_subscribe(op_id, symbol, &params) {
                        Ok(m) => m,
                        Err(e) => {
                            store.mark_deadletter(&key, &format!("build_subscribe:{e}"), now)?;
                            continue;
                        }
                    };

                    // priority: classify by op_id (private-first)
                    let p = classify_op_id_priority(op_id);
                    let prio = match p {
                        OutboundPriority::Private => OutboundPriority::Private,
                        _ => OutboundPriority::Public,
                    };

                    // subscribe limiter
                    let w = {
                        let mut lim = ws_limiter.lock().await;
                        lim.acquire_wait(prio, Instant::now())
                    };
                    if w > Duration::from_secs(0) {
                        tokio::time::sleep(w).await;
                    }

                    for m in msgs {
                        let out = outq
                            .push(
                                &cfg.exchange_id,
                                &cfg.conn_id,
                                QueuedOutbound {
                                    priority: prio,
                                    op_id: Some(op_id.to_string()),
                                    symbol: sym_opt.map(|s| s.to_string()),
                                    frame: WsOutboundFrame::Text(m.text),
                                    meta: serde_json::json!({"kind":"subscribe"}),
                                },
                                &overflow_policy,
                                now_unix_u64(),
                            )
                            .await
                            .unwrap_or(PushOutcome::Dropped);
                        stability.emit(TransportStabilityEvent::OutqOverflowOutcome {
                            exchange_id: cfg.exchange_id.clone(),
                            conn_id: cfg.conn_id.clone(),
                            outcome: map_outcome(out),
                        });
                        match out {
                            PushOutcome::Enqueued => {}
                            PushOutcome::Dropped => {
                                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id=%op_id, priority=%prio.as_str(), "outbound overflow -> dropped");
                            }
                            PushOutcome::Spilled => {
                                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id=%op_id, priority=%prio.as_str(), "outbound overflow -> spilled-to-disk");
                            }
                        }
                    }
                }
            }

            // Inbound read (bounded wait)
            let next = tokio::time::timeout(Duration::from_millis(250), read.next()).await;
            let maybe_item = next.unwrap_or_default();
            let Some(item) = maybe_item else {
                continue;
            };

            let msg = match item {
                Ok(m) => m,
                Err(e) => {
                    if e.to_string().to_ascii_lowercase().contains("rate") {
                        let mut lim = ws_limiter.lock().await;
                        let p = Duration::from_millis(250);
                        lim.apply_penalty(OutboundPriority::Private, Instant::now(), p);
                        stability.emit(TransportStabilityEvent::RlPenaltyApplied {
                            exchange_id: cfg.exchange_id.clone(),
                            conn_id: cfg.conn_id.clone(),
                            priority: OutboundPriority::Private,
                            penalty_ms: p.as_millis() as u64,
                        });
                    }
                    break;
                }
            };

            match msg {
                Message::Ping(p) => {
                    let out = outq
                        .push(
                            &cfg.exchange_id,
                            &cfg.conn_id,
                            QueuedOutbound {
                                priority: OutboundPriority::Control,
                                op_id: Some("ws.control.pong".into()),
                                symbol: None,
                                frame: WsOutboundFrame::Pong(p),
                                meta: serde_json::json!({"kind":"pong"}),
                            },
                            &overflow_policy,
                            now_unix_u64(),
                        )
                        .await
                        .unwrap_or(PushOutcome::Dropped);
                    match out {
                        PushOutcome::Enqueued => {}
                        PushOutcome::Dropped => {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.pong", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> dropped");
                        }
                        PushOutcome::Spilled => {
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.pong", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> spilled-to-disk");
                        }
                    }
                    last_inbound = Instant::now();
                    continue;
                }
                Message::Pong(_) => {
                    last_inbound = Instant::now();
                    continue;
                }
                Message::Close(_) => {
                    stability.emit(TransportStabilityEvent::ReconnectAttempt {
                        exchange_id: cfg.exchange_id.clone(),
                        conn_id: cfg.conn_id.clone(),
                        reason: ReconnectReason::CloseFrame,
                        attempt: reconnect_attempt as u64,
                    });
                    break;
                }
                Message::Text(t) => {
                    handle_inbound(
                        &adapter,
                        &cfg,
                        store,
                        &wal_tx,
                        &ws_limiter,
                        &stability,
                        t.into_bytes(),
                    )
                    .await?;
                    last_inbound = Instant::now();
                }
                Message::Binary(b) => {
                    handle_inbound(&adapter, &cfg, store, &wal_tx, &ws_limiter, &stability, b)
                        .await?;
                    last_inbound = Instant::now();
                }
                _ => {}
            }
        }

        stability.emit(TransportStabilityEvent::ConnectionState {
            exchange_id: cfg.exchange_id.clone(),
            conn_id: cfg.conn_id.clone(),
            state: ConnState::Disconnected,
        });
        stability.add_gauge("active_conn", -1);
        // reconnect path: request close, then abort tasks only if they do not join quickly
        // (best effort, because we are not in shutdown)
        outq.begin_closing();
        let out = outq
            .push(
                &cfg.exchange_id,
                &cfg.conn_id,
                QueuedOutbound::close_request(),
                &OverflowPolicy::Drop {
                    mode: DropMode::DropOldestLowPriority,
                },
                now_unix_u64(),
            )
            .await
            .unwrap_or(PushOutcome::Dropped);
        match out {
            PushOutcome::Enqueued => {}
            PushOutcome::Dropped => {
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.close", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> dropped");
            }
            PushOutcome::Spilled => {
                warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, op_id="ws.control.close", priority=%OutboundPriority::Control.as_str(), "outbound overflow -> spilled-to-disk");
            }
        }
        outq.close();

        // ensure subscriptions get resubscribed on new socket
        let _ = store.requeue_active_to_pending(&cfg.exchange_id, &cfg.conn_id, now_unix_i64());

        // join with timeout, then abort if needed
        // (this avoids runaway tasks and is safer than immediate abort)
        let mut writer = writer;
        let mut wal_writer = wal_writer;

        if tokio::time::timeout(Duration::from_secs(2), &mut writer)
            .await
            .is_err()
        {
            writer.abort();
            stability.emit(TransportStabilityEvent::ShutdownPhase {
                exchange_id: cfg.exchange_id.clone(),
                conn_id: cfg.conn_id.clone(),
                phase: ShutdownPhase::AbortTimeout,
            });
        }
        if tokio::time::timeout(Duration::from_secs(2), &mut wal_writer)
            .await
            .is_err()
        {
            wal_writer.abort();
            stability.emit(TransportStabilityEvent::ShutdownPhase {
                exchange_id: cfg.exchange_id.clone(),
                conn_id: cfg.conn_id.clone(),
                phase: ShutdownPhase::AbortTimeout,
            });
        }

        // backoff before reconnect
        let backoff = backoff_with_jitter_ms(reconnect_attempt, 200, 30_000, 250);
        reconnect_attempt = reconnect_attempt.saturating_add(1);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
    }
}

const RL_MAX_ATTEMPTS: i64 = 20;
const RL_BASE_COOLDOWN_SECS: i64 = 1;
const RL_MAX_COOLDOWN_SECS: i64 = 60;

fn looks_like_rate_limited(reason: &str) -> bool {
    let s = reason.to_ascii_lowercase();
    s.contains("rate")
        || s.contains("limit")
        || s.contains("too many")
        || s.contains("throttle")
        || s.contains("429")
        || s.contains("slow down")
}

fn rl_cooldown_secs(attempts: i64, base: i64, max: i64) -> i64 {
    // exponential backoff: base * 2^(attempts-1)
    let a = attempts.max(1).min(30) as u32;
    let mut v = base.max(1);
    // compute v * 2^(a-1) safely
    for _ in 0..(a.saturating_sub(1)) {
        v = v.saturating_mul(2);
        if v >= max {
            return max;
        }
    }
    v.min(max)
}

async fn handle_inbound(
    adapter: &Arc<dyn WsVenueAdapter>,
    cfg: &WsRunConfig,
    store: &mut SubscriptionStore,
    wal_tx: &mpsc::Sender<RawRecord>,
    ws_limiter: &Arc<Mutex<WsRateLimiter>>,
    stability: &Arc<StabilityHub>,
    raw: Vec<u8>,
) -> Result<(), String> {
    if raw.len() > cfg.max_frame_bytes {
        return Err("frame too large (DoS) -> stop".into());
    }

    let classified = adapter.classify_inbound(&raw);

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
            retry_after_ms,
            ..
        } => {
            meta.insert("reason".into(), serde_json::Value::String(reason.clone()));
            if let Some(op) = op_id.as_ref() {
                meta.insert("op_id".into(), serde_json::Value::String(op.clone()));
            }
            if let Some(s) = symbol.as_ref() {
                meta.insert("symbol".into(), serde_json::Value::String(s.clone()));
            }
            if let Some(ms) = retry_after_ms.as_ref() {
                meta.insert("retry_after_ms".into(), serde_json::Value::from(*ms));
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
        raw_bytes_b64: base64::engine::general_purpose::STANDARD.encode(&raw),
        meta: serde_json::Value::Object(meta),
    };

    // WAL queue bounded: await send (safe)
    wal_tx
        .send(rec)
        .await
        .map_err(|_| "WAL writer stopped -> stop".to_string())?;

    // Update subscription store: mark active + bump message timestamp
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
            retry_after_ms,
        } => {
            let now = now_unix_i64();
            let params_canon = params_canon_hint.as_deref().unwrap_or("{}");
            let is_rl = looks_like_rate_limited(&reason);

            if let Some(op) = op_id.clone() {
                let key = store.find_key_by_fields(
                    &cfg.exchange_id,
                    &cfg.conn_id,
                    &op,
                    symbol.as_deref(),
                    params_canon,
                )?;

                if let Some(k) = key {
                    if is_rl {
                        // attempts 上限で deadletter
                        let attempts = store.attempts_of(&k)?.unwrap_or(0);
                        if attempts >= RL_MAX_ATTEMPTS {
                            store.mark_deadletter(
                                &k,
                                &format!("nack:rate-limit:max-attempts:{reason}"),
                                now,
                            )?;
                            warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, key=%k, attempts, "rl loop -> deadletter");
                        } else {
                            // retry_after を優先し、なければ attempts に応じた backoff cooldown
                            let cooldown_secs = if let Some(ms) = retry_after_ms {
                                // ms => secs (ceil)
                                ((ms as f64) / 1000.0).ceil() as i64
                            } else {
                                rl_cooldown_secs(
                                    attempts + 1,
                                    RL_BASE_COOLDOWN_SECS,
                                    RL_MAX_COOLDOWN_SECS,
                                )
                            };

                            // pendingへ戻し、cooldownをセット
                            store.apply_rate_limit_cooldown(&k, now, cooldown_secs)?;
                            let prio = classify_op_id_priority(&op);
                            stability.emit(TransportStabilityEvent::RlCooldownSet {
                                exchange_id: cfg.exchange_id.clone(),
                                conn_id: cfg.conn_id.clone(),
                                priority: prio,
                                cooldown_secs,
                                attempts,
                            });

                            warn!(
                                exchange_id=%cfg.exchange_id,
                                conn=%cfg.conn_id,
                                key=%k,
                                attempts,
                                cooldown_secs,
                                reason=%reason,
                                "ws nack (rate-limit) -> pending + cooldown"
                            );
                        }
                    } else {
                        // RL以外は deadletter
                        store.mark_deadletter(&k, &format!("nack:{reason}"), now)?;
                        warn!(exchange_id=%cfg.exchange_id, conn=%cfg.conn_id, key=%k, reason=%reason, "ws nack (non-rate-limit) -> deadletter");
                    }
                }
            }

            // penalty auto-apply（RLの時だけ）
            if is_rl {
                let mut penalty = retry_after_ms.map(Duration::from_millis);
                if penalty.is_none() {
                    penalty = Some(Duration::from_millis(500));
                }
                if let Some(pen) = penalty {
                    let prio = match op_id.as_deref() {
                        Some(op) => classify_op_id_priority(op),
                        None => OutboundPriority::Public,
                    };
                    {
                        let mut lim = ws_limiter.lock().await;
                        lim.apply_penalty(prio, Instant::now(), pen);
                    }
                    stability.emit(TransportStabilityEvent::RlPenaltyApplied {
                        exchange_id: cfg.exchange_id.clone(),
                        conn_id: cfg.conn_id.clone(),
                        priority: prio,
                        penalty_ms: pen.as_millis() as u64,
                    });
                    warn!(
                        exchange_id=%cfg.exchange_id,
                        conn=%cfg.conn_id,
                        penalty_ms=pen.as_millis() as u64,
                        priority=%prio.as_str(),
                        reason=%reason,
                        "ws rate-limit -> applied limiter penalty"
                    );
                }
            }
        }
        _ => {}
    }

    Ok(())
}
