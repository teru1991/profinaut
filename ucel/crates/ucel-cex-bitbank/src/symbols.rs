use std::collections::BTreeMap;
use ucel_symbol_core::MarketMeta;
pub fn fetch_symbols() -> Result<Vec<String>, String> {
    Err("NotSupported".into())
}

/// NEW: MarketMeta を返す（tick/step/min_qty/min_notional）
pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    Err("NotSupported: fetch_market_meta is not implemented for this connector yet".to_string())
}
