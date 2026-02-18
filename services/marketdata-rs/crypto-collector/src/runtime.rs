use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::descriptor::{AckMatcher, ParseSection};
use crate::engine::{extract_metadata, generate_subscriptions, normalize_metadata, ParseRules, SubscriptionContext};
use crate::envelope::Envelope;
use crate::ingestion::IngestSender;
use crate::metrics::Metrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Authenticating,
    Subscribing,
    Running,
    Degraded,
}

#[derive(Debug, Clone)]
pub struct ConnectionSnapshot {
    pub state: ConnectionState,
    pub updated_at: Instant,
    pub last_error: Option<String>,
}

impl Default for ConnectionSnapshot {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            updated_at: Instant::now(),
            last_error: None,
        }
    }
}

#[derive(Default)]
pub struct StateRegistry {
    inner: tokio::sync::Mutex<HashMap<String, ConnectionSnapshot>>,
}

impl StateRegistry {
    pub fn set(&self, key: impl Into<String>, state: ConnectionState, err: Option<String>) {
        self.inner.lock().expect("poisoned").insert(
            key.into(),
            ConnectionSnapshot {
                state,
                updated_at: Instant::now(),
                last_error: err,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<ConnectionSnapshot> {
        self.inner.lock().expect("poisoned").get(key).cloned()
    }
}

pub struct BackoffPolicy {
    pub base_ms: u64,
    pub cap_ms: u64,
    pub jitter_ms: u64,
    rng: ChaCha8Rng,
}

impl BackoffPolicy {
    pub fn seeded(base_ms: u64, cap_ms: u64, jitter_ms: u64, seed: u64) -> Self {
        Self { base_ms, cap_ms, jitter_ms, rng: ChaCha8Rng::seed_from_u64(seed) }
    }

    pub fn next_delay_ms(&mut self, attempt: u32) -> u64 {
        let exp = self.base_ms.saturating_mul(1_u64 << attempt.min(16));
        let capped = exp.min(self.cap_ms);
        let jitter = if self.jitter_ms == 0 { 0 } else { self.rng.gen_range(0..=self.jitter_ms) };
        capped.saturating_add(jitter)
    }
}

#[derive(Debug, Clone)]
pub struct UrlRotator {
    urls: Vec<String>,
    idx: usize,
}

impl UrlRotator {
    pub fn new(urls: Vec<String>) -> Self {
        Self { urls, idx: 0 }
    }

    pub fn current(&self) -> Option<&str> {
        self.urls.get(self.idx).map(|s| s.as_str())
    }

    pub fn rotate(&mut self) -> Option<&str> {
        if self.urls.is_empty() {
            return None;
        }
        self.idx = (self.idx + 1) % self.urls.len();
        self.current()
    }
}

pub fn parse_ws_payload(raw: &[u8], is_binary: bool) -> Result<Value, serde_json::Error> {
    if !is_binary {
        return serde_json::from_slice(raw);
    }

    if let Ok(text) = std::str::from_utf8(raw) {
        return serde_json::from_str(text);
    }

    Ok(json!({
        "binary_base64": base64::engine::general_purpose::STANDARD.encode(raw),
        "binary_len": raw.len()
    }))
}

#[derive(Debug, Error)]
pub enum AckError {
    #[error("ack timeout")]
    Timeout,
}

pub struct AckGate {
    matcher: Option<AckMatcher>,
    correlation_pointer: Option<String>,
    expected: HashSet<String>,
    pub acked: HashSet<String>,
}

impl AckGate {
    pub fn new(matcher: Option<AckMatcher>, correlation_pointer: Option<String>, expected: HashSet<String>) -> Self {
        Self { matcher, correlation_pointer, expected, acked: HashSet::new() }
    }

    pub fn on_message(&mut self, payload: &Value) {
        let Some(m) = &self.matcher else { return; };
        let field = payload.pointer(&m.field).and_then(|v| v.as_str()).unwrap_or_default();
        if field != m.value {
            return;
        }

        if let Some(ptr) = &self.correlation_pointer {
            if let Some(corr) = payload.pointer(ptr).and_then(|v| v.as_str()) {
                if self.expected.contains(corr) {
                    self.acked.insert(corr.to_string());
                }
            }
        } else {
            self.acked = self.expected.clone();
        }
    }

    pub fn is_complete(&self) -> bool {
        self.expected.is_subset(&self.acked)
    }

    pub async fn wait_until<F>(&mut self, timeout: Duration, mut next: F) -> Result<(), AckError>
    where
        F: FnMut() -> Option<Value>,
    {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(msg) = next() {
                self.on_message(&msg);
                if self.is_complete() {
                    return Ok(());
                }
            } else {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        Err(AckError::Timeout)
    }
}

pub fn build_envelope(
    payload: &Value,
    parse_rules: &ParseSection,
    maps: Option<&crate::maps::NormalizationMaps>,
    exchange: &str,
    conn_id: &str,
    local_time_ns: u64,
) -> Result<Envelope, crate::engine::EngineError> {
    let rules = ParseRules {
        channel_pointer: parse_rules.channel.clone(),
        symbol_pointer: parse_rules.symbol.clone(),
        server_time_pointer: parse_rules.server_time.clone(),
        sequence_pointer: parse_rules.sequence.clone(),
        message_id_pointer: parse_rules.message_id.clone(),
        expr_enabled: false,
        expressions: Vec::new(),
        expr_config: Default::default(),
    };
    let extracted = extract_metadata(payload, &rules)?;
    let default_maps = crate::maps::NormalizationMaps::default();
    let normalized = normalize_metadata(&extracted, maps.unwrap_or(&default_maps));

    let mut builder = Envelope::builder(
        "descriptor-runtime@1.4",
        conn_id,
        exchange,
        normalized.symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
        normalized.channel.unwrap_or_else(|| "unknown".to_string()),
        payload.clone(),
    )
    .local_time_ns(local_time_ns);

    if let Some(v) = normalized.server_time.and_then(|v| v.as_i64()) {
        builder = builder.server_time(v);
    }
    if let Some(v) = normalized.sequence.and_then(|v| v.as_u64()) {
        builder = builder.sequence(v);
    }
    if let Some(v) = normalized
        .message_id
        .and_then(|v| v.as_str().map(|s| s.to_string()))
    {
        builder = builder.message_id(v);
    }

    Ok(builder.build())
}

pub fn send_envelope(sender: &IngestSender, metrics: &Metrics, envelope: Envelope) {
    let exchange = envelope.exchange.clone();
    metrics.set_ws_connected(&exchange, 1);
    if sender.try_send(envelope).is_err() {
        metrics.inc_ingest_errors_total(&exchange);
    }
}

pub struct InstanceSupervisor {
    tasks: Vec<JoinHandle<()>>,
    pub states: Arc<StateRegistry>,
}

impl InstanceSupervisor {
    pub fn new() -> Self {
        Self { tasks: Vec::new(), states: Arc::new(StateRegistry::default()) }
    }

    pub fn spawn_guarded<F>(&mut self, key: String, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let states_for_task = self.states.clone();
        let states_for_monitor = self.states.clone();
        let key_for_task = key.clone();
        let key_for_monitor = key.clone();
        let main = tokio::spawn(async move {
            states_for_task.set(key_for_task.clone(), ConnectionState::Connecting, None);
            task.await;
            states_for_task.set(key_for_task, ConnectionState::Disconnected, None);
        });
        self.tasks.push(tokio::spawn(async move {
            if let Err(err) = main.await {
                states_for_monitor.set(key_for_monitor, ConnectionState::Degraded, Some(format!("panic detected: {err}")));
            }
        }));
    }

    pub async fn join_all(self) {
        for t in self.tasks {
            let _ = t.await;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeQualityTracker {
    pub total: u64,
    pub with_server_time: u64,
    pub clock_skew_ms: Vec<f64>,
    pub end_to_end_lag_ms: Vec<f64>,
}

impl TimeQualityTracker {
    pub fn record(&mut self, server_time: Option<i64>, local_time_ns: u64) {
        self.total += 1;
        if let Some(st) = server_time {
            self.with_server_time += 1;
            let local_ms = local_time_ns as f64 / 1_000_000.0;
            let skew = local_ms - st as f64;
            self.clock_skew_ms.push(skew);
            self.end_to_end_lag_ms.push(skew.max(0.0));
        }
    }

    pub fn presence_ratio(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.with_server_time as f64 / self.total as f64 }
    }
}

pub fn generate_subscribe_messages(source: &str, symbols: Vec<String>, channels: Vec<String>, conn_id: &str) -> Result<Vec<String>, crate::engine::EngineError> {
    let ctx = SubscriptionContext {
        symbols,
        channels,
        conn_id: conn_id.to_string(),
        args: HashMap::new(),
        max_outputs: 100_000,
    };
    generate_subscriptions(source, &ctx, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::ParseSection;

    #[test]
    fn deterministic_backoff_with_seed() {
        let mut a = BackoffPolicy::seeded(10, 200, 5, 7);
        let mut b = BackoffPolicy::seeded(10, 200, 5, 7);
        assert_eq!(a.next_delay_ms(0), b.next_delay_ms(0));
        assert_eq!(a.next_delay_ms(1), b.next_delay_ms(1));
        assert_eq!(a.next_delay_ms(2), b.next_delay_ms(2));
    }

    #[test]
    fn url_rotation_cycles() {
        let mut r = UrlRotator::new(vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(r.current(), Some("a"));
        assert_eq!(r.rotate(), Some("b"));
        assert_eq!(r.rotate(), Some("c"));
        assert_eq!(r.rotate(), Some("a"));
    }

    #[tokio::test]
    async fn ack_gate_matches_by_correlation() {
        let matcher = AckMatcher { field: "/type".into(), value: "subscribed".into(), correlation_pointer: None, timeout_ms: 5000 };
        let mut gate = AckGate::new(
            Some(matcher),
            Some("/id".into()),
            ["s1".to_string(), "s2".to_string()].into_iter().collect(),
        );
        let mut msgs = vec![json!({"type":"subscribed","id":"s1"}), json!({"type":"subscribed","id":"s2"})].into_iter();
        gate.wait_until(Duration::from_millis(50), || msgs.next()).await.unwrap();
        assert!(gate.is_complete());
    }

    #[tokio::test]
    async fn ack_gate_timeout() {
        let matcher = AckMatcher { field: "/type".into(), value: "subscribed".into(), correlation_pointer: None, timeout_ms: 5000 };
        let mut gate = AckGate::new(Some(matcher), Some("/id".into()), ["x".to_string()].into_iter().collect());
        let mut msgs = vec![json!({"type":"noop"})].into_iter();
        let err = gate.wait_until(Duration::from_millis(5), || msgs.next()).await.unwrap_err();
        matches!(err, AckError::Timeout);
    }

    #[test]
    fn extraction_and_normalize_to_envelope() {
        let payload = json!({"topic":"trade","s":"XBTUSD","ts":1000,"seq":7,"id":"m1"});
        let parse = ParseSection {
            channel: "/topic".into(),
            symbol: "/s".into(),
            server_time: Some("/ts".into()),
            sequence: Some("/seq".into()),
            message_id: Some("/id".into()),
            expr: None,
        };
        let mut maps = crate::maps::NormalizationMaps::default();
        maps.symbol_map.insert("XBTUSD".into(), "BTCUSD".into());
        maps.channel_map.insert("trade".into(), "trades".into());
        let env = build_envelope(&payload, &parse, Some(&maps), "bitmex", "conn-1", 123).unwrap();
        assert_eq!(env.symbol, "BTCUSD");
        assert_eq!(env.channel, "trades");
        assert_eq!(env.server_time, Some(1000));
        assert_eq!(env.sequence, Some(7));
    }
}
