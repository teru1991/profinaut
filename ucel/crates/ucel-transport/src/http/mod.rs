//! HTTP transport helpers (rate limit + retry).
//!
//! This module provides a wrapper that can be placed around any `Transport`
//! implementation to make HTTP calls more robust:
//! - Per-venue bucket rate limits
//! - Private/public separation (auth calls prioritized)
//! - Retry with exponential backoff and optional Retry-After

pub mod limiter;
pub mod retry;

use std::sync::Arc;
use std::time::Instant;

use crate::{
    enforce_auth_boundary, HttpRequest, HttpResponse, RequestContext, RetryPolicy, Transport,
};

use limiter::{HttpRateLimiter, HttpRateLimiterConfig};
use retry::{decide_retry, RetryDecision};
use ucel_core::UcelError;

#[derive(Debug, Clone)]
pub struct ReliableHttpConfig {
    pub limiter: HttpRateLimiterConfig,
    pub retry: RetryPolicy,
    pub max_attempts: u32,
}

impl Default for ReliableHttpConfig {
    fn default() -> Self {
        Self {
            limiter: HttpRateLimiterConfig::default(),
            retry: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 30_000,
                jitter_ms: 200,
                respect_retry_after: true,
            },
            max_attempts: 5,
        }
    }
}

/// A wrapper that upgrades `send_http` to be rate-limit aware and retryable.
///
/// `connect_ws` is forwarded unchanged.
#[derive(Clone)]
pub struct ReliableTransport<T> {
    inner: Arc<T>,
    limiter: Arc<HttpRateLimiter>,
    cfg: ReliableHttpConfig,
}

impl<T> ReliableTransport<T>
where
    T: Transport + Send + Sync + 'static,
{
    pub fn new(inner: Arc<T>, cfg: ReliableHttpConfig) -> Self {
        let limiter = Arc::new(HttpRateLimiter::new(cfg.limiter.clone()));
        Self {
            inner,
            limiter,
            cfg,
        }
    }

    async fn send_http_inner(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        enforce_auth_boundary(&ctx)?;

        let mut attempt: u32 = 0;
        let mut last_err: Option<UcelError> = None;

        while attempt < self.cfg.max_attempts.max(1) {
            let w = self
                .limiter
                .acquire_wait(&ctx.venue, ctx.requires_auth, Instant::now())
                .await;
            if w.as_nanos() > 0 {
                tokio::time::sleep(w).await;
            }

            match self.inner.send_http(req.clone(), ctx.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    last_err = Some(err.clone());
                    match decide_retry(&self.cfg.retry, attempt, &err) {
                        RetryDecision::DoNotRetry => return Err(err),
                        RetryDecision::RetryAfter(d) => {
                            tokio::time::sleep(d).await;
                        }
                    }
                }
            }

            attempt = attempt.saturating_add(1);
        }

        Err(last_err
            .unwrap_or_else(|| UcelError::new(ucel_core::ErrorCode::Internal, "retry exhausted")))
    }
}

#[allow(async_fn_in_trait)]
impl<T> Transport for ReliableTransport<T>
where
    T: Transport + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.send_http_inner(req, ctx).await
    }

    async fn connect_ws(
        &self,
        req: crate::WsConnectRequest,
        ctx: RequestContext,
    ) -> Result<crate::WsStream, UcelError> {
        self.inner.connect_ws(req, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTransport;

    #[allow(async_fn_in_trait)]
    impl Transport for FakeTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Err(
                ucel_core::UcelError::new(ucel_core::ErrorCode::RateLimited, "rl")
                    .with_retry_after_ms(1),
            )
        }
        async fn connect_ws(
            &self,
            _req: crate::WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<crate::WsStream, UcelError> {
            Ok(crate::WsStream { connected: true })
        }
    }

    fn ctx() -> RequestContext {
        RequestContext {
            trace_id: "t".into(),
            request_id: "r".into(),
            run_id: "run".into(),
            op: ucel_core::OpName::FetchTicker,
            venue: "x".into(),
            policy_id: "p".into(),
            key_id: None,
            requires_auth: false,
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reliable_transport_retries_until_exhausted() {
        let t = Arc::new(FakeTransport);
        let cfg = ReliableHttpConfig {
            max_attempts: 2,
            ..Default::default()
        };
        let rt = ReliableTransport::new(t, cfg);
        let req = HttpRequest {
            path: "/".into(),
            method: "GET".into(),
            body: None,
        };

        let err = rt.send_http(req, ctx()).await.unwrap_err();
        assert_eq!(err.code, ucel_core::ErrorCode::RateLimited);
    }
}
