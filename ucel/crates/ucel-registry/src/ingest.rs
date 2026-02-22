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

#[derive(Debug, Clone)]
pub struct DefaultIngestDriver {
    exchange_id: &'static str,
}

impl DefaultIngestDriver {
    pub fn new(exchange_id: &'static str) -> Self {
        Self { exchange_id }
    }
}

#[async_trait]
impl ExchangeIngestDriver for DefaultIngestDriver {
    fn exchange_id(&self) -> &'static str {
        self.exchange_id
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        match self.exchange_id {
            "sbivc" => Err("NotSupported".to_string()),
            _ => Ok(vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()]),
        }
    }

    async fn run_ws_ingest(
        &self,
        _plan: IngestPlanRef,
        _runtime: IngestRuntimeRef,
        _rules: IngestRulesRef,
        _cfg: IngestConfigRef,
    ) -> Result<(), String> {
        Ok(())
    }
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

pub fn default_drivers() -> Vec<DefaultIngestDriver> {
    registered_ingest_driver_ids()
        .into_iter()
        .map(DefaultIngestDriver::new)
        .collect()
}

pub fn find_default_driver(exchange_id: &str) -> Option<DefaultIngestDriver> {
    registered_ingest_driver_ids()
        .into_iter()
        .find(|id| *id == exchange_id)
        .map(DefaultIngestDriver::new)
}
