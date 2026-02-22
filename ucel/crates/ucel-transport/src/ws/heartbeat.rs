#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    pub ping_interval_secs: u64,
    pub idle_timeout_secs: u64,
}

pub fn is_idle(last_message_age_secs: u64, cfg: HeartbeatConfig) -> bool {
    last_message_age_secs >= cfg.idle_timeout_secs
}
