use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum SupportLevel {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "not_supported")]
    NotSupported,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatePolicy {
    pub messages_per_second: Option<u32>,
    pub messages_per_hour: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatPolicy {
    pub ping_interval_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
    pub max_connection_age_secs: Option<u64>, // NEW: preemptive rotate (24h cut etc)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SafetyProfile {
    pub max_streams_per_conn: Option<usize>,
    pub max_symbols_per_conn: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeWsRules {
    pub exchange_id: String,
    pub support_level: SupportLevel,

    pub rate: Option<RatePolicy>,
    pub heartbeat: Option<HeartbeatPolicy>,
    pub entitlement: Option<String>,

    pub safety_profile: Option<SafetyProfile>,

    pub stability: Option<StabilityRules>,

    // legacy overrides
    pub max_streams_per_conn: Option<usize>,
    pub max_symbols_per_conn: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityRules {
    pub buckets: Option<StabilityBuckets>,
    pub rate_limit: Option<StabilityRateLimit>,
    pub circuit_breaker: Option<StabilityCircuitBreaker>,
    pub overflow: Option<StabilityOverflow>,
    pub stale: Option<StabilityStale>,
    pub graceful: Option<StabilityGraceful>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityBuckets {
    pub control_rps: Option<f64>,
    pub private_rps: Option<f64>,
    pub public_rps: Option<f64>,
    pub min_gap_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityRateLimit {
    pub max_attempts: Option<i64>,
    pub base_cooldown_secs: Option<i64>,
    pub max_cooldown_secs: Option<i64>,
    pub default_penalty_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityCircuitBreaker {
    pub failure_threshold: Option<u32>,
    pub success_threshold: Option<u32>,
    pub cooldown_ms: Option<u64>,
    pub half_open_max_trials: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityOverflow {
    pub mode: Option<String>,
    pub slowdown_max_wait_ms: Option<u64>,
    pub spill_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityStale {
    pub stale_after_secs: Option<u64>,
    pub sweep_interval_ms: Option<u64>,
    pub max_batch: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityGraceful {
    pub drain_timeout_ms: Option<u64>,
    pub join_timeout_ms: Option<u64>,
}

impl ExchangeWsRules {
    pub fn effective_max_streams_per_conn(&self) -> usize {
        self.max_streams_per_conn
            .or_else(|| {
                self.safety_profile
                    .as_ref()
                    .and_then(|p| p.max_streams_per_conn)
            })
            .unwrap_or(25)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityRules {
    pub buckets: Option<StabilityBuckets>,
    pub rate_limit: Option<StabilityRateLimit>,
    pub circuit_breaker: Option<StabilityCircuitBreaker>,
    pub overflow: Option<StabilityOverflow>,
    pub stale: Option<StabilityStale>,
    pub graceful: Option<StabilityGraceful>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityBuckets {
    pub control_rps: Option<f64>,
    pub private_rps: Option<f64>,
    pub public_rps: Option<f64>,
    pub min_gap_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityRateLimit {
    pub max_attempts: Option<i64>,
    pub base_cooldown_secs: Option<i64>,
    pub max_cooldown_secs: Option<i64>,
    pub default_penalty_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityCircuitBreaker {
    pub failure_threshold: Option<u32>,
    pub success_threshold: Option<u32>,
    pub cooldown_ms: Option<u64>,
    pub half_open_max_trials: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityOverflow {
    pub mode: Option<String>,
    pub slowdown_max_wait_ms: Option<u64>,
    pub spill_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityStale {
    pub stale_after_secs: Option<u64>,
    pub sweep_interval_ms: Option<u64>,
    pub max_batch: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StabilityGraceful {
    pub drain_timeout_ms: Option<u64>,
    pub join_timeout_ms: Option<u64>,
}
