use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HubConfig {
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub ws_buffer: usize,
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
            max_retries: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 1500,
            ws_buffer: 128,
        }
    }
}
