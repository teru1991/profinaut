use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use ucel_core::{ErrorCode, OpName, UcelError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    pub path: String,
    pub method: String,
    pub body: Option<Bytes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsConnectRequest {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WsStream {
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    pub trace_id: String,
    pub request_id: String,
    pub run_id: String,
    pub op: OpName,
    pub venue: String,
    pub policy_id: String,
    pub key_id: Option<String>,
    pub requires_auth: bool,
}

#[allow(async_fn_in_trait)]
pub trait Transport {
    async fn send_http(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError>;
    async fn connect_ws(
        &self,
        req: WsConnectRequest,
        ctx: RequestContext,
    ) -> Result<WsStream, UcelError>;
}

#[derive(Debug)]
pub struct WsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
}

impl WsBackpressure {
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx }
    }

    pub fn sender(&self) -> mpsc::Sender<Bytes> {
        self.tx.clone()
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Default)]
pub struct SubscriptionBook {
    subscriptions: HashMap<String, String>,
}

impl SubscriptionBook {
    pub fn upsert(&mut self, channel_key: String, symbol: String) {
        self.subscriptions.insert(channel_key, symbol);
    }

    pub fn remove(&mut self, channel_key: &str) {
        self.subscriptions.remove(channel_key);
    }

    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Retryable,
    NonRetryable,
}

pub fn classify_error(code: &ErrorCode) -> RetryClass {
    match code {
        ErrorCode::Timeout
        | ErrorCode::Network
        | ErrorCode::Upstream5xx
        | ErrorCode::RateLimited => RetryClass::Retryable,
        _ => RetryClass::NonRetryable,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_ms: u64,
    pub respect_retry_after: bool,
}

pub fn next_retry_delay_ms(policy: &RetryPolicy, attempt: u32, retry_after_ms: Option<u64>) -> u64 {
    if policy.respect_retry_after {
        if let Some(delay) = retry_after_ms {
            return delay.min(policy.max_delay_ms);
        }
    }
    let exp = policy
        .base_delay_ms
        .saturating_mul(2u64.saturating_pow(attempt));
    (exp + policy.jitter_ms).min(policy.max_delay_ms)
}

pub fn enforce_auth_boundary(ctx: &RequestContext) -> Result<(), UcelError> {
    if ctx.requires_auth && ctx.key_id.is_none() {
        return Err(UcelError::new(
            ErrorCode::MissingAuth,
            "private operation requires key_id",
        ));
    }
    Ok(())
}

pub trait TransportMetricsSink {
    fn http_requests_total(&self, venue: &str, op: OpName, status: u16);
    fn http_retries_total(&self, venue: &str, op: OpName, reason: &str);
    fn ws_reconnect_total(&self, venue: &str);
    fn rate_limited_total(&self, venue: &str, op: OpName);
    fn health_update(&self, venue: &str, health: HealthStatus);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub level: HealthLevel,
    pub degraded_reason: Option<String>,
    pub last_success_ts: Option<u64>,
    pub last_error_code: Option<ErrorCode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_classification_matches_policy() {
        assert_eq!(classify_error(&ErrorCode::Timeout), RetryClass::Retryable);
        assert_eq!(
            classify_error(&ErrorCode::InvalidOrder),
            RetryClass::NonRetryable
        );
    }

    #[test]
    fn retry_after_is_respected() {
        let p = RetryPolicy {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            jitter_ms: 5,
            respect_retry_after: true,
        };
        assert_eq!(next_retry_delay_ms(&p, 2, Some(333)), 333);
    }

    #[test]
    fn private_op_without_key_is_rejected() {
        let ctx = RequestContext {
            trace_id: "t".into(),
            request_id: "r".into(),
            run_id: "run".into(),
            op: OpName::PlaceOrder,
            venue: "x".into(),
            policy_id: "p".into(),
            key_id: None,
            requires_auth: true,
        };
        let err = enforce_auth_boundary(&ctx).unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[test]
    fn public_op_does_not_require_key() {
        let ctx = RequestContext {
            trace_id: "t".into(),
            request_id: "r".into(),
            run_id: "run".into(),
            op: OpName::FetchTicker,
            venue: "x".into(),
            policy_id: "p".into(),
            key_id: None,
            requires_auth: false,
        };
        assert!(enforce_auth_boundary(&ctx).is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_ws_channel_enforces_backpressure() {
        let mut queue = WsBackpressure::with_capacity(1);
        let sender = queue.sender();

        sender.try_send(Bytes::from_static(b"msg1")).unwrap();
        assert!(sender.try_send(Bytes::from_static(b"msg2")).is_err());

        let msg = queue.recv().await.unwrap();
        assert_eq!(msg, Bytes::from_static(b"msg1"));
    }

    #[test]
    fn subscription_book_updates_in_o1_map() {
        let mut book = SubscriptionBook::default();
        book.upsert("ticker:BTC".into(), "BTC".into());
        book.upsert("ticker:ETH".into(), "ETH".into());
        book.remove("ticker:BTC");
        assert_eq!(book.len(), 1);
    }
}
