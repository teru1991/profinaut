use super::config::HubConfig;
use super::errors::HubError;
use super::registry::SpecRegistry;
use super::{ExchangeId, OperationKey};
use crate::policy::enforce_surface_for_catalog_entry;
use bytes::Bytes;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, OnceLock};
use tokio::time::{sleep, Duration};
use ucel_transport::security::{EndpointAllowlist, SubdomainPolicy};
use ucel_transport::{next_retry_delay_ms, RetryPolicy};

#[derive(Clone)]
pub struct RestHub {
    exchange: ExchangeId,
    client: reqwest::Client,
    config: Arc<HubConfig>,
}

pub struct RestResponse {
    pub status: u16,
    pub body: Bytes,
}

impl RestResponse {
    pub fn json_value(&self) -> Result<Value, HubError> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn json_typed<T: DeserializeOwned>(&self) -> Result<T, HubError> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

fn bounded_retry_delay_ms(policy: &RetryPolicy, attempt: u32, retry_after_ms: Option<u64>) -> u64 {
    next_retry_delay_ms(policy, attempt, retry_after_ms)
}

impl RestHub {
    pub(crate) fn new(
        exchange: ExchangeId,
        client: reqwest::Client,
        config: Arc<HubConfig>,
    ) -> Self {
        Self {
            exchange,
            client,
            config,
        }
    }

    pub async fn call(
        &self,
        op_key: impl Into<OperationKey>,
        params: Option<&[(&str, &str)]>,
        body: Option<Value>,
    ) -> Result<RestResponse, HubError> {
        let key = op_key.into();
        let spec = SpecRegistry::global()?.resolve_rest(self.exchange, &key)?;
        enforce_surface_for_catalog_entry(self.exchange.as_str(), spec)
            .map_err(|e| HubError::RegistryValidation(e.to_string()))?;
        let url = format!(
            "{}{}",
            spec.base_url.clone().unwrap_or_default(),
            spec.path.clone().unwrap_or_default()
        );
        let method = spec.method.clone().unwrap_or_else(|| "GET".to_string());
        validate_https_endpoint(self.exchange, &spec.base_url.clone().unwrap_or_default())?;

        let retry_policy = RetryPolicy {
            base_delay_ms: self.config.base_backoff_ms,
            max_delay_ms: self.config.max_backoff_ms,
            jitter_ms: 0,
            respect_retry_after: true,
        };

        let mut attempt = 0;
        loop {
            let mut request = self.client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                &url,
            );
            if let Some(params) = params {
                request = request.query(params);
            }
            if let Some(body) = body.clone() {
                request = request.json(&body);
            }

            let resp = request.timeout(self.config.request_timeout).send().await?;
            if resp.status().as_u16() != 429 && !resp.status().is_server_error() {
                let status = resp.status().as_u16();
                let body = resp.bytes().await?;
                return Ok(RestResponse { status, body });
            }

            if attempt >= self.config.max_retries {
                let status = resp.status().as_u16();
                let body = resp.bytes().await?;
                return Ok(RestResponse { status, body });
            }

            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000);

            let mut delay = bounded_retry_delay_ms(&retry_policy, attempt, retry_after);
            let jitter = rand::thread_rng().gen_range(0..=20);
            delay += jitter;
            sleep(Duration::from_millis(delay)).await;
            attempt += 1;
        }
    }

    pub async fn call_vendor_public_typed(
        &self,
        operation_id: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<ucel_core::VendorPublicRestTypedEnvelope, HubError> {
        let operation = vendor_public_rest_extension_operation(self.exchange, operation_id)?;
        let response = self.call(operation_id.to_string(), params, None).await?;
        let body = response.json_value()?;
        ucel_core::build_vendor_public_rest_typed_envelope(
            self.exchange.as_str(),
            operation_id,
            &operation.path_or_channel,
            &body,
        )
        .map_err(|e| HubError::RegistryValidation(e.to_string()))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DomesticPublicInventory {
    entries: Vec<DomesticPublicInventoryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomesticPublicInventoryEntry {
    venue: String,
    api_kind: String,
    public_id: String,
    path_or_channel: String,
    surface_class: String,
}

fn domestic_public_inventory() -> Result<&'static DomesticPublicInventory, HubError> {
    static INVENTORY: OnceLock<Result<DomesticPublicInventory, HubError>> = OnceLock::new();
    INVENTORY
        .get_or_init(|| {
            serde_json::from_str::<DomesticPublicInventory>(include_str!(
                "../../../../../ucel/coverage_v2/domestic_public/jp_public_inventory.json"
            ))
            .map_err(HubError::Json)
        })
        .as_ref()
        .map_err(|e| HubError::RegistryValidation(e.to_string()))
}

pub fn list_vendor_public_rest_extension_operation_ids(
    exchange: ExchangeId,
) -> Result<Vec<String>, HubError> {
    let mut ids = domestic_public_inventory()?
        .entries
        .iter()
        .filter(|entry| {
            entry.venue == exchange.as_str()
                && entry.api_kind == "rest"
                && entry.surface_class == "vendor_public_extension"
        })
        .map(|entry| entry.public_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

fn vendor_public_rest_extension_operation(
    exchange: ExchangeId,
    operation_id: &str,
) -> Result<DomesticPublicInventoryEntry, HubError> {
    domestic_public_inventory()?
        .entries
        .iter()
        .find(|entry| {
            entry.venue == exchange.as_str()
                && entry.api_kind == "rest"
                && entry.surface_class == "vendor_public_extension"
                && entry.public_id == operation_id
        })
        .cloned()
        .ok_or_else(|| HubError::UnknownOperation {
            exchange: exchange.as_str().to_string(),
            key: operation_id.to_string(),
        })
}

fn rest_allowlist(exchange: ExchangeId) -> Result<EndpointAllowlist, HubError> {
    let hosts: Vec<&str> = match exchange {
        ExchangeId::Binance => vec!["api.binance.com"],
        ExchangeId::BinanceUsdm => vec!["fapi.binance.com"],
        ExchangeId::BinanceCoinm => vec!["dapi.binance.com"],
        ExchangeId::BinanceOptions => vec!["eapi.binance.com"],
        ExchangeId::Bitbank => vec!["api.bitbank.cc"],
        ExchangeId::Bitflyer => vec!["api.bitflyer.com"],
        ExchangeId::Bitget => vec!["api.bitget.com"],
        ExchangeId::Bithumb => vec!["api.bithumb.com"],
        ExchangeId::Bitmex => vec!["www.bitmex.com"],
        ExchangeId::Bittrade => vec!["api.bittrade.co.jp"],
        ExchangeId::Bybit => vec!["api.bybit.com", "api-testnet.bybit.com"],
        ExchangeId::Coinbase => vec!["api.exchange.coinbase.com", "api.coinbase.com"],
        ExchangeId::Coincheck => vec!["coincheck.com", "api.coincheck.com"],
        ExchangeId::Deribit => vec!["www.deribit.com", "test.deribit.com"],
        ExchangeId::Gmocoin => vec!["api.coin.z.com"],
        ExchangeId::Htx => vec!["api.htx.com", "api.huobi.pro"],
        ExchangeId::Kraken => vec!["api.kraken.com"],
        ExchangeId::Okx => vec!["www.okx.com", "aws.okx.com"],
        ExchangeId::Sbivc => vec!["api.sbivc.co.jp"],
        ExchangeId::Upbit => vec!["api.upbit.com"],
    };
    EndpointAllowlist::new(hosts, SubdomainPolicy::Exact)
        .map_err(|e| HubError::RegistryValidation(e.message))
}

fn validate_https_endpoint(exchange: ExchangeId, base: &str) -> Result<(), HubError> {
    let al = rest_allowlist(exchange)?;
    let u = al
        .validate_https_wss(base)
        .map_err(|e| HubError::RegistryValidation(e.message))?;
    if u.scheme() != "https" {
        return Err(HubError::RegistryValidation(
            "rest base_url must be https".to_string(),
        ));
    }
    Ok(())
}

pub fn private_rest_operation_from_catalog_id(id: &str) -> Option<ucel_core::PrivateRestOperation> {
    let id = id.to_ascii_lowercase();
    if !id.contains("private") {
        return None;
    }
    if id.contains("balance") || id.contains("assets") {
        Some(ucel_core::PrivateRestOperation::GetBalances)
    } else if id.contains("openorders") || id.contains("open_orders") {
        Some(ucel_core::PrivateRestOperation::GetOpenOrders)
    } else if id.contains("cancel") {
        Some(ucel_core::PrivateRestOperation::CancelOrder)
    } else if id.contains("fills") || id.contains("matchresults") || id.contains("executions") {
        Some(ucel_core::PrivateRestOperation::GetFills)
    } else if id.contains("position") {
        Some(ucel_core::PrivateRestOperation::GetPositions)
    } else if id.contains("order") {
        Some(ucel_core::PrivateRestOperation::GetOrder)
    } else if id.contains("account") || id.contains("profile") {
        Some(ucel_core::PrivateRestOperation::GetAccountProfile)
    } else {
        None
    }
}

pub fn public_rest_operation_from_catalog_id(id: &str) -> Option<ucel_core::MarketDataChannel> {
    let id = id.to_ascii_lowercase();
    if id.contains("ticker") {
        Some(ucel_core::MarketDataChannel::Ticker)
    } else if id.contains("trade") {
        Some(ucel_core::MarketDataChannel::Trades)
    } else if id.contains("book") || id.contains("orderbook") {
        Some(ucel_core::MarketDataChannel::OrderBook)
    } else if id.contains("candle") || id.contains("kline") {
        Some(ucel_core::MarketDataChannel::Candles)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_is_bounded() {
        let p = RetryPolicy {
            base_delay_ms: 100,
            max_delay_ms: 500,
            jitter_ms: 0,
            respect_retry_after: true,
        };
        assert_eq!(bounded_retry_delay_ms(&p, 10, None), 500);
        assert_eq!(bounded_retry_delay_ms(&p, 0, Some(2000)), 500);
    }

    #[test]
    fn private_op_mapper_is_stable() {
        assert_eq!(
            private_rest_operation_from_catalog_id("private.rest.order.cancel.post"),
            Some(ucel_core::PrivateRestOperation::CancelOrder)
        );
        assert_eq!(
            private_rest_operation_from_catalog_id("public.rest.market.ticker"),
            None
        );
        assert_eq!(
            public_rest_operation_from_catalog_id("public.rest.market.orderbook"),
            Some(ucel_core::MarketDataChannel::OrderBook)
        );
    }
}
