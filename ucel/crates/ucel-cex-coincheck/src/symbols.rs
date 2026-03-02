use std::collections::BTreeMap;
use ucel_market_meta_catalog as catalog;
use ucel_symbol_core::{Exchange, MarketMeta, Snapshot};

const EXCHANGE_CONST: Exchange = Exchange::Coincheck;

pub fn fetch_symbols() -> Result<Vec<String>, String> {
    let snap = catalog::snapshot_for_exchange(EXCHANGE_CONST);
    if snap.instruments.is_empty() {
        return Err("catalog empty for exchange=Coincheck".to_string());
    }
    Ok(snap.instruments.into_iter().map(|i| i.raw_symbol).collect())
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let snap = catalog::snapshot_for_exchange(EXCHANGE_CONST);
    if snap.instruments.is_empty() {
        return Err("catalog empty for exchange=Coincheck".to_string());
    }
    Ok(snap)
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snap = fetch_symbol_snapshot().await?;
    let mut out = BTreeMap::new();
    for s in snap.instruments {
        let mm = catalog::get_meta(EXCHANGE_CONST, s.market_type.clone(), &s.raw_symbol)
            .ok_or_else(|| {
                format!(
                    "catalog missing meta exchange=Coincheck symbol={}",
                    s.raw_symbol
                )
            })?;
        out.insert(s.raw_symbol, mm);
    }
    Ok(out)
}
