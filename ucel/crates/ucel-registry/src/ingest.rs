use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct IngestPlanRef {
    pub exchange_id: String,
    pub seed_len: usize,
}

#[derive(Debug, Clone)]
pub struct IngestRuntimeRef {
    pub store_path: String,
    pub journal_dir: String,
}

#[derive(Debug, Clone)]
pub struct IngestRulesRef {
    pub support_level: String,
}

#[derive(Debug, Clone)]
pub struct IngestConfigRef {
    pub enable_private_ws: bool,
}

#[async_trait]
pub trait ExchangeIngestDriver: Send + Sync {
    fn exchange_id(&self) -> &'static str;
    async fn fetch_symbols(&self) -> Result<Vec<String>, String>;
    async fn run_ws_ingest(
        &self,
        _plan: IngestPlanRef,
        _runtime: IngestRuntimeRef,
        _rules: IngestRulesRef,
        _cfg: IngestConfigRef,
    ) -> Result<(), String>;
}

pub fn registered_ingest_driver_ids() -> Vec<&'static str> {
    vec![
        "binance",
        "binance-coinm",
        "binance-options",
        "binance-usdm",
        "bitbank",
        "bitflyer",
        "bitget",
        "bitmex",
        "bittrade",
        "bybit",
        "coinbase",
        "coincheck",
        "deribit",
        "gmocoin",
        "htx",
        "kraken",
        "okx",
        "sbivc",
        "upbit",
    ]
}
