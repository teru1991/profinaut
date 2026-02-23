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

    pub max_streams_per_conn: Option<usize>,
    pub max_symbols_per_conn: Option<usize>,
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
