use serde::Deserialize;
use std::collections::BTreeMap;
use ucel_core::Decimal;
use ucel_symbol_core::{Exchange, MarketMeta, MarketMetaId, MarketType};

const PUBLIC_BASE: &str = "https://api.coin.z.com/public";

#[derive(Debug, Deserialize)]
struct ApiResp<T> {
    status: u16,
    #[serde(default)]
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct SymbolRow {
    symbol: String,
    min_order_size: String,
    max_order_size: String,
    size_step: String,
    tick_size: String,
    taker_fee: String,
    maker_fee: String,
}

pub async fn fetch_symbols() -> Result<Vec<String>, String> {
    let url = format!("{PUBLIC_BASE}/v1/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("gmocoin symbols http status={}", resp.status()));
    }

    let body: ApiResp<Vec<SymbolRow>> = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != 0 {}

    let mut out: Vec<String> = body.data.into_iter().map(|r| r.symbol).collect();
    out.sort();
    out.dedup();
    Ok(out)
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let url = format!("{PUBLIC_BASE}/v1/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("gmocoin symbols http status={}", resp.status()));
    }

    let body: ApiResp<Vec<SymbolRow>> = resp.json().await.map_err(|e| e.to_string())?;

    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();
    for r in body.data {
        let tick = r.tick_size.parse::<Decimal>().map_err(|e| e.to_string())?;
        let step = r.size_step.parse::<Decimal>().map_err(|e| e.to_string())?;
        let min_qty = r
            .min_order_size
            .parse::<Decimal>()
            .map_err(|e| e.to_string())?;

        let canonical = format!("{}/JPY", r.symbol);
        let mm = MarketMeta {
            min_qty: Some(min_qty),
            min_notional: None,
            ..MarketMeta::new(
                MarketMetaId::new(Exchange::Gmocoin, MarketType::Spot, r.symbol),
                tick,
                step,
            )
        };
        mm.validate_basic()
            .map_err(|e| format!("gmocoin invalid meta {canonical}: {e}"))?;
        out.insert(canonical, mm);
    }
    Ok(out)
}
