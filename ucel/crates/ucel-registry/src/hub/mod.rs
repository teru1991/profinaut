//! Single-entry REST + WS Hub.
//!
//! Quick start:
//! ```no_run
//! use ucel_registry::hub::{ExchangeId, Hub};
//!
//! #[tokio::main]
//! async fn main() {
//!     let hub = Hub::default();
//!     let _ = hub.list_operations(ExchangeId::Binance).unwrap();
//! }
//! ```

pub mod config;
pub mod errors;
pub mod registry;
pub mod rest;
pub mod ws;

pub use config::HubConfig;
pub use errors::HubError;
pub use registry::SpecRegistry;
pub use rest::{RestHub, RestResponse};
pub use ws::{WsHub, WsMessage};

use std::str::FromStr;
use std::sync::Arc;

pub type OperationKey = String;
pub type ChannelKey = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ExchangeId {
    Binance,
    Bybit,
    Coinbase,
    Coincheck,
    Deribit,
    Gmocoin,
    Kraken,
    Okx,
    Upbit,
}

impl ExchangeId {
    pub fn as_str(self) -> &'static str {
        match self {
            ExchangeId::Binance => "binance",
            ExchangeId::Bybit => "bybit",
            ExchangeId::Coinbase => "coinbase",
            ExchangeId::Coincheck => "coincheck",
            ExchangeId::Deribit => "deribit",
            ExchangeId::Gmocoin => "gmocoin",
            ExchangeId::Kraken => "kraken",
            ExchangeId::Okx => "okx",
            ExchangeId::Upbit => "upbit",
        }
    }
}

impl FromStr for ExchangeId {
    type Err = HubError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "binance" => Ok(ExchangeId::Binance),
            "bybit" => Ok(ExchangeId::Bybit),
            "coinbase" => Ok(ExchangeId::Coinbase),
            "coincheck" => Ok(ExchangeId::Coincheck),
            "deribit" => Ok(ExchangeId::Deribit),
            "gmocoin" => Ok(ExchangeId::Gmocoin),
            "kraken" => Ok(ExchangeId::Kraken),
            "okx" => Ok(ExchangeId::Okx),
            "upbit" => Ok(ExchangeId::Upbit),
            other => Err(HubError::UnknownExchange(other.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct Hub {
    client: reqwest::Client,
    config: Arc<HubConfig>,
}

impl Hub {
    pub fn new(config: HubConfig) -> Result<Self, HubError> {
        let _ = SpecRegistry::global()?;
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .build()?;
        Ok(Self {
            client,
            config: Arc::new(config),
        })
    }

    pub fn rest(&self, exchange: ExchangeId) -> RestHub {
        RestHub::new(exchange, self.client.clone(), self.config.clone())
    }

    pub fn ws(&self, exchange: ExchangeId) -> WsHub {
        WsHub::new(exchange, self.config.clone())
    }

    pub fn list_operations(&self, exchange: ExchangeId) -> Result<Vec<OperationKey>, HubError> {
        Ok(SpecRegistry::global()?.list_operations(exchange))
    }

    pub fn list_channels(&self, exchange: ExchangeId) -> Result<Vec<ChannelKey>, HubError> {
        Ok(SpecRegistry::global()?.list_channels(exchange))
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new(HubConfig::default()).expect("hub default init")
    }
}
