use crate::config::HttpConfig;
use crate::errors::{UcelIrError, UcelIrErrorKind};
use reqwest::{Client, RequestBuilder, Response};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct HttpClient {
    inner: Client,
    config: HttpConfig,
    limiter: Mutex<TokenBucket>,
}

impl HttpClient {
    pub fn new(config: HttpConfig) -> Result<Self, UcelIrError> {
        config.validate()?;
        let inner = Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(config.timeout())
            .build()
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Http, e.to_string()))?;

        Ok(Self {
            inner,
            limiter: Mutex::new(TokenBucket::new(config.rate_limit_per_sec)),
            config,
        })
    }

    pub fn inner(&self) -> &Client {
        &self.inner
    }

    pub async fn send_with_retry(
        &self,
        mut request_factory: impl FnMut(&Client) -> RequestBuilder,
    ) -> Result<Response, UcelIrError> {
        let mut last_err: Option<UcelIrError> = None;
        for attempt in 0..=self.config.max_retries {
            self.wait_for_permit().await?;
            let result = request_factory(&self.inner).send().await;
            match result {
                Ok(response) if response.status().as_u16() == 429 => {
                    last_err = Some(UcelIrError::new(
                        UcelIrErrorKind::RateLimit,
                        "upstream status 429",
                    ));
                }
                Ok(response) if response.status().is_server_error() => {
                    last_err = Some(UcelIrError::new(
                        UcelIrErrorKind::Upstream,
                        format!("upstream status {}", response.status().as_u16()),
                    ));
                }
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_err = Some(UcelIrError::new(UcelIrErrorKind::Http, err.to_string()));
                }
            }

            if attempt < self.config.max_retries {
                let delay = self.retry_delay(attempt);
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_err
            .unwrap_or_else(|| UcelIrError::new(UcelIrErrorKind::Internal, "missing HTTP error")))
    }

    async fn wait_for_permit(&self) -> Result<(), UcelIrError> {
        loop {
            let wait_for = {
                let mut limiter = self.limiter.lock().map_err(|_| {
                    UcelIrError::new(UcelIrErrorKind::Internal, "rate limiter lock poisoned")
                })?;
                limiter.try_take()
            };

            if let Some(duration) = wait_for {
                tokio::time::sleep(duration).await;
                continue;
            }
            return Ok(());
        }
    }

    fn retry_delay(&self, attempt: u32) -> Duration {
        let exp = 1_u64.checked_shl(attempt.min(16)).unwrap_or(u64::MAX);
        let backoff = self.config.base_backoff_ms.saturating_mul(exp);
        let jitter = (attempt as u64 * 17) % self.config.base_backoff_ms;
        Duration::from_millis(backoff.saturating_add(jitter))
    }
}

struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(rate_limit_per_sec: u32) -> Self {
        Self {
            capacity: rate_limit_per_sec,
            tokens: rate_limit_per_sec as f64,
            refill_per_sec: rate_limit_per_sec as f64,
            last_refill: Instant::now(),
        }
    }

    fn try_take(&mut self) -> Option<Duration> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity as f64);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            None
        } else {
            let seconds = (1.0 - self.tokens) / self.refill_per_sec;
            Some(Duration::from_secs_f64(seconds.max(0.001)))
        }
    }
}
