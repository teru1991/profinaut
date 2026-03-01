use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ucel_symbol_core::{MarketMetaSnapshot, MarketType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketMetaConnectorCapabilities {
    pub supports_rest_snapshot: bool,
    pub supports_incremental_rest: bool,
    pub market_types: Vec<MarketType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketMetaRateLimitPolicy {
    pub max_inflight: usize,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub jitter: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MarketMetaContext {
    pub request_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum MarketMetaAdapterError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("mapping error: {0}")]
    Mapping(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

#[async_trait]
pub trait MarketMetaFetcher: Send + Sync {
    fn capabilities(&self) -> MarketMetaConnectorCapabilities;
    fn rate_limit_policy(&self) -> MarketMetaRateLimitPolicy;

    async fn fetch_market_meta_snapshot(
        &self,
        ctx: &MarketMetaContext,
    ) -> Result<MarketMetaSnapshot, MarketMetaAdapterError>;
}
