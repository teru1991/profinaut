use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// ucel/coverage のディレクトリ（SSOT）
    pub coverage_dir: PathBuf,

    /// ucel-ws-rules の rules ディレクトリ（例: ucel/crates/ucel-ws-rules/rules）
    pub rules_dir: String,

    /// SubscriptionStore の sqlite パス
    pub store_path: String,

    /// WAL出力ディレクトリ（ucel-journal）
    pub journal_dir: String,

    /// WAL最大サイズ（bytes）
    pub wal_max_bytes: u64,

    /// fsync mode（ucel-journal）
    pub fsync_mode: ucel_journal::FsyncMode,

    /// WS受信/送信キュー容量
    pub recv_queue_cap: usize,

    /// 最大フレームサイズ（DoS耐性）
    pub max_frame_bytes: usize,

    /// drip購読（inflight）上限
    pub max_inflight_per_conn: usize,

    /// WS connect timeout
    pub connect_timeout: Duration,

    /// idle timeout（無受信→ping/reconnect）
    pub idle_timeout: Duration,

    /// reconnect storm guard
    pub reconnect_storm_window: Duration,
    pub reconnect_storm_max: usize,

    /// 1取引所あたりの最大接続数（安全停止ガード）
    pub max_connections_per_exchange: usize,

    /// private ws を有効化（v1ではfalse推奨）
    pub enable_private_ws: bool,

    /// 起動対象取引所のallowlist（未指定なら coverage 全部→v1は supervisor 側で gmocoin に絞る）
    pub exchange_allowlist: Option<Vec<String>>,
}

impl IngestConfig {
    pub fn from_env() -> Result<Self, String> {
        let coverage_dir = env_path("UCEL_COVERAGE_DIR", "../../coverage");
        let rules_dir = env_string("UCEL_RULES_DIR", "ucel/crates/ucel-ws-rules/rules");
        let store_path = env_string("UCEL_STORE_PATH", "ucel-ws-subscriber.sqlite");
        let journal_dir = env_string("UCEL_JOURNAL_DIR", ".ucel-wal");

        let wal_max_bytes = env_u64("UCEL_WAL_MAX_BYTES", 256 * 1024 * 1024)?;
        let fsync_mode = env_fsync_mode("UCEL_FSYNC_MODE", "balanced");

        let recv_queue_cap = env_usize("UCEL_RECV_QUEUE_CAP", 4096)?;
        let max_frame_bytes = env_usize("UCEL_MAX_FRAME_BYTES", 4 * 1024 * 1024)?;
        let max_inflight_per_conn = env_usize("UCEL_MAX_INFLIGHT_PER_CONN", 64)?;

        let connect_timeout = Duration::from_secs(env_u64("UCEL_CONNECT_TIMEOUT_SECS", 10)?);
        let idle_timeout = Duration::from_secs(env_u64("UCEL_IDLE_TIMEOUT_SECS", 30)?);

        let reconnect_storm_window =
            Duration::from_secs(env_u64("UCEL_RECONNECT_STORM_WINDOW_SECS", 30)?);
        let reconnect_storm_max = env_usize("UCEL_RECONNECT_STORM_MAX", 12)?;

        let max_connections_per_exchange = env_usize("UCEL_MAX_CONNECTIONS_PER_EXCHANGE", 512)?;
        let enable_private_ws = env_bool("UCEL_ENABLE_PRIVATE_WS", false);

        let exchange_allowlist = env_opt_csv("UCEL_EXCHANGE_ALLOWLIST");

        Ok(Self {
            coverage_dir,
            rules_dir,
            store_path,
            journal_dir,
            wal_max_bytes,
            fsync_mode,
            recv_queue_cap,
            max_frame_bytes,
            max_inflight_per_conn,
            connect_timeout,
            idle_timeout,
            reconnect_storm_window,
            reconnect_storm_max,
            max_connections_per_exchange,
            enable_private_ws,
            exchange_allowlist,
        })
    }
}

fn env_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_path(key: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(key).unwrap_or_else(|_| default.to_string()))
}

fn env_u64(key: &str, default: u64) -> Result<u64, String> {
    match std::env::var(key) {
        Ok(v) => v
            .parse::<u64>()
            .map_err(|e| format!("{key} parse error: {e}")),
        Err(_) => Ok(default),
    }
}

fn env_usize(key: &str, default: usize) -> Result<usize, String> {
    match std::env::var(key) {
        Ok(v) => v
            .parse::<usize>()
            .map_err(|e| format!("{key} parse error: {e}")),
        Err(_) => Ok(default),
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(
            v.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        ),
        Err(_) => default,
    }
}

fn env_opt_csv(key: &str) -> Option<Vec<String>> {
    std::env::var(key).ok().map(|v| {
        v.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn env_fsync_mode(key: &str, default: &str) -> ucel_journal::FsyncMode {
    let v = std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .to_lowercase();
    match v.as_str() {
        "safe" | "every" | "every_record" => ucel_journal::FsyncMode::SafeEveryRecord,
        "balanced" => ucel_journal::FsyncMode::Balanced,
        "relaxed" | "none" => ucel_journal::FsyncMode::Balanced,
        _ => ucel_journal::FsyncMode::Balanced,
    }
}
