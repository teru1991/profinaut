use std::time::Duration;

#[derive(Debug, Clone)]
pub struct InvokerConfig {
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub ws_buffer: usize,
    pub ws_max_reconnects: u32,
}

impl Default for InvokerConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
            max_retries: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 1_500,
            ws_buffer: 128,
            ws_max_reconnects: 3,
        }
    }
}
