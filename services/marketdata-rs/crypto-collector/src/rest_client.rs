use std::sync::Arc;
use std::time::Duration;

use governor::{clock::DefaultClock, state::InMemoryState, Quota, RateLimiter};
use reqwest::{Method, RequestBuilder, StatusCode};
use thiserror::Error;

use crate::runtime::ConnectionState;

#[derive(Debug, Clone)]
pub struct RestRuntimeConfig {
    pub base_urls: Vec<String>,
    pub requests_per_second: u32,
    pub max_retries: u32,
    pub secret_env: Option<String>,
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("no base urls configured")]
    NoBaseUrls,
    #[error("missing required secret env var: {0}")]
    MissingSecret(String),
    #[error("all urls failed: {0}")]
    Exhausted(String),
}

#[derive(Clone)]
pub struct RestClient {
    http: reqwest::Client,
    cfg: RestRuntimeConfig,
    limiter: Arc<RateLimiter<governor::state::NotKeyed, InMemoryState, DefaultClock>>,
    pub state: ConnectionState,
}

impl RestClient {
    pub fn new(cfg: RestRuntimeConfig) -> Result<Self, RestError> {
        if cfg.base_urls.is_empty() {
            return Err(RestError::NoBaseUrls);
        }
        let q = Quota::per_second(std::num::NonZeroU32::new(cfg.requests_per_second.max(1)).unwrap());
        Ok(Self {
            http: reqwest::Client::new(),
            cfg,
            limiter: Arc::new(RateLimiter::direct(q)),
            state: ConnectionState::Disconnected,
        })
    }

    fn sign(&self, req: RequestBuilder) -> Result<RequestBuilder, RestError> {
        if let Some(var) = &self.cfg.secret_env {
            let secret = std::env::var(var).map_err(|_| RestError::MissingSecret(var.clone()))?;
            Ok(req.header("X-Signature", format!("env:{secret}")))
        } else {
            Ok(req)
        }
    }

    pub async fn execute(&self, method: Method, path: &str) -> Result<String, RestError> {
        let mut last_err = String::new();
        for base in &self.cfg.base_urls {
            let url = format!("{}{}", base.trim_end_matches('/'), path);
            for attempt in 0..=self.cfg.max_retries {
                self.limiter.until_ready().await;
                let req = self.http.request(method.clone(), &url);
                let req = self.sign(req)?;
                match req.send().await {
                    Ok(resp) if is_retryable(resp.status()) => {
                        last_err = format!("retryable status {}", resp.status());
                        if attempt < self.cfg.max_retries {
                            let mut backoff = crate::runtime::BackoffPolicy::seeded(
                                20,   // base_ms
                                1000, // cap_ms (1 second)
                                100,  // jitter_ms
                                rand::thread_rng().gen::<u64>(),
                            );
                            let delay = backoff.next_delay_ms(attempt);
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                        continue;
                    }
                    Ok(resp) => {
                        return resp.text().await.map_err(|e| RestError::Exhausted(e.to_string()));
                    }
                    Err(err) => {
                        last_err = err.to_string();
                        if attempt < self.cfg.max_retries {
                            let mut backoff = crate::runtime::BackoffPolicy::seeded(
                                20,   // base_ms
                                1000, // cap_ms (1 second)
                                100,  // jitter_ms
                                rand::thread_rng().gen::<u64>(),
                            );
                            let delay = backoff.next_delay_ms(attempt);
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                    }
                }
            }
        }
        Err(RestError::Exhausted(last_err))
    }
}

fn is_retryable(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_secret_degrades() {
        let c = RestClient::new(RestRuntimeConfig {
            base_urls: vec!["http://localhost".into()],
            requests_per_second: 1,
            max_retries: 1,
            secret_env: Some("MISSING_TEST_SECRET_E".into()),
        })
        .unwrap();
        let err = c.sign(c.http.request(Method::GET, "http://localhost")).unwrap_err();
        matches!(err, RestError::MissingSecret(_));
    }
}
