#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub repo_root: String,
    pub coverage_dir: String,
    pub journal_dir: String,
    pub store_path: String,
    pub rules_dir: String,
    pub enable_private_ws: bool,
    pub exchange_allowlist: Option<Vec<String>>,
    pub default_symbols: Vec<String>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            repo_root: "..".into(),
            coverage_dir: "../../coverage".into(),
            journal_dir: "./.ucel-wal".into(),
            store_path: "./.ucel-store.sqlite3".into(),
            rules_dir: "./rules".into(),
            enable_private_ws: false,
            exchange_allowlist: None,
            default_symbols: vec!["BTC/USDT".into(), "ETH/USDT".into()],
        }
    }
}
