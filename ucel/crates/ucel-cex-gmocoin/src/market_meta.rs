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

const PUBLIC_BASE: &str = "https://api.coin.z.com/public";

#[derive(Debug, Deserialize)]
struct ApiResp<T> {
    status: u16,
    #[serde(default)]
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolRow {
    symbol: String,
    min_order_size: String,
    max_order_size: String,
    size_step: String,
    tick_size: String,
}

fn parse_dec(s: &str, field: &'static str) -> Result<Decimal, MarketMetaAdapterError> {
    Decimal::from_str(s)
        .map_err(|_| MarketMetaAdapterError::Mapping(format!("invalid_decimal:{field}")))
}

pub fn parse_market_meta_snapshot(
    json: &str,
) -> Result<MarketMetaSnapshot, MarketMetaAdapterError> {
    let body: ApiResp<Vec<SymbolRow>> =
        serde_json::from_str(json).map_err(|e| MarketMetaAdapterError::Mapping(e.to_string()))?;
    let _ = body.status;

    let mut markets = Vec::with_capacity(body.data.len());
    for r in body.data {
        let id = MarketMetaId::new(Exchange::Gmocoin, MarketType::Spot, r.symbol);
        let tick = parse_dec(&r.tick_size, "tick_size")?;
        let step = parse_dec(&r.size_step, "size_step")?;
        let min_qty = parse_dec(&r.min_order_size, "min_order_size")?;
        let max_qty = parse_dec(&r.max_order_size, "max_order_size")?;

        let mut meta = MarketMeta::new(id, tick, step);
        meta.min_qty = Some(min_qty);
        meta.max_qty = Some(max_qty);

        meta.validate_meta()
            .map_err(|e| MarketMetaAdapterError::Mapping(format!("validate_meta:{e}")))?;
        markets.push(meta);
    }

    Ok(MarketMetaSnapshot::new_rest(markets))
}

pub struct GmoCoinMarketMetaFetcher {
    client: reqwest::Client,
}

impl GmoCoinMarketMetaFetcher {
    pub fn new() -> Result<Self, MarketMetaAdapterError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))?;
        Ok(Self { client })
    }

    async fn fetch_json(&self) -> Result<String, MarketMetaAdapterError> {
        let url = format!("{PUBLIC_BASE}/v1/symbols");
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(MarketMetaAdapterError::Transport(format!(
                "gmocoin http status={}",
                resp.status()
            )));
        }
        resp.text()
            .await
            .map_err(|e| MarketMetaAdapterError::Transport(e.to_string()))
    }
}

#[async_trait]
impl MarketMetaFetcher for GmoCoinMarketMetaFetcher {
    fn capabilities(&self) -> MarketMetaConnectorCapabilities {
        MarketMetaConnectorCapabilities {
            supports_rest_snapshot: true,
            supports_incremental_rest: false,
            market_types: vec![MarketType::Spot],
        }
    }

    fn rate_limit_policy(&self) -> MarketMetaRateLimitPolicy {
        MarketMetaRateLimitPolicy {
            max_inflight: 2,
            base_backoff_ms: 200,
            max_backoff_ms: 5_000,
            jitter: true,
        }
    }

    async fn fetch_market_meta_snapshot(
        &self,
        _ctx: &MarketMetaContext,
    ) -> Result<MarketMetaSnapshot, MarketMetaAdapterError> {
        let json = self.fetch_json().await?;
        parse_market_meta_snapshot(&json)
    }
}
