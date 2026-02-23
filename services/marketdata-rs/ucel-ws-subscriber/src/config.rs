use std::time::Duration;

use ucel_journal::FsyncMode;

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub journal_dir: String,
    pub store_path: String,
    pub rules_dir: String,
    pub enable_private_ws: bool,
    pub exchange_allowlist: Option<Vec<String>>,
    pub wal_max_bytes: u64,
    pub fsync_mode: FsyncMode,
    pub max_connections_per_exchange: usize,
    pub recv_queue_cap: usize,
    pub max_frame_bytes: usize,
    pub max_inflight_per_conn: usize,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub reconnect_storm_window: Duration,
    pub reconnect_storm_max: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            journal_dir: "./.ucel-wal".into(),
            store_path: "./.ucel-store.sqlite3".into(),
            rules_dir: "./ucel/crates/ucel-ws-rules/rules".into(),
            enable_private_ws: false,
            exchange_allowlist: None,
            wal_max_bytes: 256 * 1024 * 1024,
            fsync_mode: FsyncMode::Balanced,
            max_connections_per_exchange: 128,
            recv_queue_cap: 4096,
            max_frame_bytes: 4 * 1024 * 1024,
            max_inflight_per_conn: 64,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
            reconnect_storm_window: Duration::from_secs(60),
            reconnect_storm_max: 20,
        }
    }
}
