use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::time::Duration;
use ucel_symbol_adapter::market_meta::{
    MarketMetaAdapterError, MarketMetaConnectorCapabilities, MarketMetaContext, MarketMetaFetcher,
    MarketMetaRateLimitPolicy,
};
use ucel_symbol_core::{Exchange, MarketMeta, MarketMetaId, MarketMetaSnapshot, MarketType};

use crate::symbols::to_canonical_symbol;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolInfo {
    status: String,
    base_asset: String,
    quote_asset: String,
    #[serde(default)]
    filters: Vec<Filter>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
#[allow(clippy::enum_variant_names)] // Mirrors Binance API filter names for lossless serde mapping.
enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter { tick_size: String },
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize {
        min_qty: String,
        max_qty: String,
        step_size: String,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional { min_notional: String },
    #[serde(other)]
    Other,
}

fn d(s: &str, field: &'static str) -> Result<Decimal, MarketMetaAdapterError> {
    Decimal::from_str(s)
        .map_err(|_| MarketMetaAdapterError::Mapping(format!("invalid_decimal:{field}")))
}

pub fn parse_market_meta_snapshot(
    json: &str,
) -> Result<MarketMetaSnapshot, MarketMetaAdapterError> {
    let info: ExchangeInfo =
        serde_json::from_str(json).map_err(|e| MarketMetaAdapterError::Mapping(e.to_string()))?;
    let mut markets = Vec::new();

    for s in info.symbols {
        if s.status != "TRADING" {
            continue;
        }

        let mut tick: Option<Decimal> = None;
        let mut step: Option<Decimal> = None;
        let mut min_qty: Option<Decimal> = None;
        let mut max_qty: Option<Decimal> = None;
        let mut min_notional: Option<Decimal> = None;

        for f in s.filters {
            match f {
                Filter::PriceFilter { tick_size } => tick = Some(d(&tick_size, "tick_size")?),
                Filter::LotSize {
                    min_qty: mn,
                    max_qty: mx,
                    step_size,
                } => {
                    min_qty = Some(d(&mn, "min_qty")?);
                    max_qty = Some(d(&mx, "max_qty")?);
                    step = Some(d(&step_size, "step_size")?);
                }
                Filter::MinNotional { min_notional: mn } => {
                    min_notional = Some(d(&mn, "min_notional")?);
                }
                Filter::Other => {}
            }
        }

        let tick = tick.ok_or_else(|| {
            MarketMetaAdapterError::Mapping("missing:PRICE_FILTER.tickSize".into())
        })?;
        let step = step
            .ok_or_else(|| MarketMetaAdapterError::Mapping("missing:LOT_SIZE.stepSize".into()))?;

        let raw_symbol = to_canonical_symbol(&s.base_asset, &s.quote_asset);
        let id = MarketMetaId::new(Exchange::Binance, MarketType::Spot, raw_symbol);

        let mut meta = MarketMeta::new(id, tick, step);
        meta.base = Some(s.base_asset);
        meta.quote = Some(s.quote_asset);
        meta.min_qty = min_qty;
        meta.max_qty = max_qty;
        meta.min_notional = min_notional;

        meta.validate_meta()
            .map_err(|e| MarketMetaAdapterError::Mapping(format!("validate_meta:{e}")))?;

        markets.push(meta);
    }

    Ok(MarketMetaSnapshot::new_rest(markets))
}

pub struct BinanceSpotMarketMetaFetcher {
    client: reqwest::Client,
}

impl BinanceSpotMarketMetaFetcher {
    pub fn new() -> Result<Self, MarketMetaAdapterError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))?;
        Ok(Self { client })
    }

    async fn fetch_exchange_info_json(&self) -> Result<String, MarketMetaAdapterError> {
        let url = "https://api.binance.com/api/v3/exchangeInfo";
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(MarketMetaAdapterError::Transport(format!(
                "binance exchangeInfo http status={}",
                resp.status()
            )));
        }
        resp.text()
            .await
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))
    }
}

#[async_trait]
impl MarketMetaFetcher for BinanceSpotMarketMetaFetcher {
    fn capabilities(&self) -> MarketMetaConnectorCapabilities {
        MarketMetaConnectorCapabilities {
            supports_rest_snapshot: true,
            supports_incremental_rest: false,
            market_types: vec![MarketType::Spot],
        }
    }

    fn rate_limit_policy(&self) -> MarketMetaRateLimitPolicy {
        MarketMetaRateLimitPolicy {
            max_inflight: 1,
            base_backoff_ms: 200,
            max_backoff_ms: 10_000,
            jitter: true,
        }
    }

    async fn fetch_market_meta_snapshot(
        &self,
        _ctx: &MarketMetaContext,
    ) -> Result<MarketMetaSnapshot, MarketMetaAdapterError> {
        let json = self.fetch_exchange_info_json().await?;
        parse_market_meta_snapshot(&json)
    }
}
