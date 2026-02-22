use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub rest_interval_ms: u64,
    pub enable_rest: bool,
    pub enable_ws: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub exchanges: Vec<ExchangeConfig>,
}
