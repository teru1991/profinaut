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

use crate::default_capabilities_for_residency;
use crate::policy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use ucel_core::{Capabilities, VenueAccessScope};

pub type OperationKey = String;
pub type ChannelKey = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExchangeId {
    Binance,
    BinanceUsdm,
    BinanceCoinm,
    BinanceOptions,
    Bitbank,
    Bitflyer,
    Bitget,
    Bithumb,
    Bitmex,
    Bittrade,
    Bybit,
    Coinbase,
    Coincheck,
    Deribit,
    Gmocoin,
    Htx,
    Kraken,
    Okx,
    Sbivc,
    Upbit,
}

impl ExchangeId {
    pub const fn all() -> &'static [ExchangeId] {
        &[
            ExchangeId::Binance,
            ExchangeId::BinanceUsdm,
            ExchangeId::BinanceCoinm,
            ExchangeId::BinanceOptions,
            ExchangeId::Bitbank,
            ExchangeId::Bitflyer,
            ExchangeId::Bitget,
            ExchangeId::Bithumb,
            ExchangeId::Bitmex,
            ExchangeId::Bittrade,
            ExchangeId::Bybit,
            ExchangeId::Coinbase,
            ExchangeId::Coincheck,
            ExchangeId::Deribit,
            ExchangeId::Gmocoin,
            ExchangeId::Htx,
            ExchangeId::Kraken,
            ExchangeId::Okx,
            ExchangeId::Sbivc,
            ExchangeId::Upbit,
        ]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            ExchangeId::Binance => "binance",
            ExchangeId::BinanceUsdm => "binance-usdm",
            ExchangeId::BinanceCoinm => "binance-coinm",
            ExchangeId::BinanceOptions => "binance-options",
            ExchangeId::Bitbank => "bitbank",
            ExchangeId::Bitflyer => "bitflyer",
            ExchangeId::Bitget => "bitget",
            ExchangeId::Bithumb => "bithumb",
            ExchangeId::Bitmex => "bitmex",
            ExchangeId::Bittrade => "bittrade",
            ExchangeId::Bybit => "bybit",
            ExchangeId::Coinbase => "coinbase",
            ExchangeId::Coincheck => "coincheck",
            ExchangeId::Deribit => "deribit",
            ExchangeId::Gmocoin => "gmocoin",
            ExchangeId::Htx => "htx",
            ExchangeId::Kraken => "kraken",
            ExchangeId::Okx => "okx",
            ExchangeId::Sbivc => "sbivc",
            ExchangeId::Upbit => "upbit",
        }
    }
}

impl FromStr for ExchangeId {
    type Err = HubError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        registry::find_registration(s)
            .map(|r| r.exchange_id)
            .ok_or_else(|| HubError::UnknownExchange(s.to_ascii_lowercase()))
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

    pub fn list_exchanges(&self) -> Vec<ExchangeId> {
        registry::list_registered_exchanges()
    }

    pub fn exchange_exists(&self, exchange: &str) -> bool {
        registry::find_registration(exchange).is_some()
    }

    pub fn list_catalog_entries(&self, exchange: ExchangeId) -> Result<(usize, usize), HubError> {
        SpecRegistry::global()?.catalog_entries_counts(exchange)
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

    pub fn venue_access_scope(&self, exchange: ExchangeId) -> Result<VenueAccessScope, HubError> {
        policy::scope_for_venue(exchange.as_str())
            .map_err(|e| HubError::RegistryValidation(e.to_string()))
    }

    pub fn capabilities(&self, exchange: ExchangeId) -> Result<Capabilities, HubError> {
        default_capabilities_for_residency(exchange.as_str())
            .map_err(|e| HubError::RegistryValidation(e.to_string()))
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new(HubConfig::default()).expect("hub default init")
    }
}
