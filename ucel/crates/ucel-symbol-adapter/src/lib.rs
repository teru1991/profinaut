pub mod market_meta;

use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use ucel_symbol_core::{MarketMeta, MarketType, Snapshot};
pub use ucel_symbol_store::SymbolEvent;

pub type SymbolEventStream = Pin<Box<dyn Stream<Item = SymbolEvent> + Send>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMetaRow {
    pub canonical_symbol: String,
    pub meta: MarketMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMetaSnapshot {
    pub ts_recv: std::time::SystemTime,
    pub market_type: Option<MarketType>,
    pub rows: Vec<MarketMetaRow>,
}

pub fn market_meta_from_snapshot(snapshot: &Snapshot) -> MarketMetaSnapshot {
    let mut rows = Vec::with_capacity(snapshot.instruments.len());
    for si in &snapshot.instruments {
        let canonical_symbol = format!("{}/{}", si.base, si.quote);
        rows.push(MarketMetaRow {
            canonical_symbol,
            meta: MarketMeta::from(si),
        });
    }
    MarketMetaSnapshot {
        ts_recv: std::time::SystemTime::now(),
        market_type: None,
        rows,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingQuality {
    Exact,
    Partial,
    BestEffort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorCapabilities {
    pub supports_rest_snapshot: bool,
    pub supports_ws_events: bool,
    pub supports_incremental_rest: bool,
    pub market_types: Vec<MarketType>,
    pub symbol_status_mapping_quality: MappingQuality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitPolicy {
    pub max_inflight: usize,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub jitter: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolContext {
    pub request_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum SymbolAdapterError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("mapping error: {0}")]
    Mapping(String),
}

#[async_trait]
pub trait SymbolFetcher: Send + Sync {
    fn capabilities(&self) -> ConnectorCapabilities;
    fn rate_limit_policy(&self) -> RateLimitPolicy;
    async fn fetch_snapshot(&self, ctx: &SymbolContext) -> Result<Snapshot, SymbolAdapterError>;
}

#[async_trait]
pub trait MarketMetaFetcher: Send + Sync {
    fn capabilities(&self) -> ConnectorCapabilities;
    fn rate_limit_policy(&self) -> RateLimitPolicy;

    async fn fetch_market_meta_snapshot(
        &self,
        ctx: &SymbolContext,
    ) -> Result<MarketMetaSnapshot, SymbolAdapterError>;
}

#[async_trait]
pub trait SymbolSubscriber: Send + Sync {
    fn capabilities(&self) -> ConnectorCapabilities;
    fn rate_limit_policy(&self) -> RateLimitPolicy;
    async fn subscribe_events(
        &self,
        ctx: &SymbolContext,
    ) -> Result<SymbolEventStream, SymbolAdapterError>;
}
