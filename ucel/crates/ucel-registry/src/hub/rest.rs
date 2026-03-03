use super::config::HubConfig;
use super::errors::HubError;
use super::registry::SpecRegistry;
use super::{ExchangeId, OperationKey};
use bytes::Bytes;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
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
}

fn rest_allowlist(exchange: ExchangeId) -> Result<EndpointAllowlist, HubError> {
    let hosts: Vec<&str> = match exchange {
        ExchangeId::Binance => vec!["api.binance.com", "fapi.binance.com", "dapi.binance.com"],
        ExchangeId::Bybit => vec!["api.bybit.com", "api-testnet.bybit.com"],
        ExchangeId::Coinbase => vec!["api.exchange.coinbase.com", "api.coinbase.com"],
        ExchangeId::Coincheck => vec!["coincheck.com", "api.coincheck.com"],
        ExchangeId::Deribit => vec!["www.deribit.com", "test.deribit.com"],
        ExchangeId::Gmocoin => vec!["api.coin.z.com"],
        ExchangeId::Kraken => vec!["api.kraken.com"],
        ExchangeId::Okx => vec!["www.okx.com", "aws.okx.com"],
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
}
