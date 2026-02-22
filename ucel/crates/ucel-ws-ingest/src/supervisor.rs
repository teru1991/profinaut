use crate::config::IngestConfig;

pub async fn run_supervisor(cfg: &IngestConfig) -> Result<Vec<String>, String> {
    let mut exchanges = vec![
        "binance".to_string(),
        "bybit".to_string(),
        "okx".to_string(),
        "sbivc".to_string(),
    ];
    if let Some(allow) = &cfg.exchange_allowlist {
        exchanges.retain(|x| allow.contains(x));
    }
    Ok(exchanges)
}
