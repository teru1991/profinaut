use serde::Deserialize;
use std::collections::BTreeMap;
use ucel_symbol_core::MarketMeta;

#[derive(Debug, Deserialize)]
struct Resp {
    #[serde(default)]
    data: Vec<Item>,
}
#[derive(Debug, Deserialize)]
struct Item {
    #[serde(default, rename = "instId")]
    inst_id: String,
    #[serde(default, rename = "instType")]
    inst_type: String,
}

pub fn to_canonical_symbol(inst_id: &str) -> String { inst_id.replace('-', "/") }
pub fn to_exchange_symbol(canonical: &str) -> String { canonical.replace('/', "-") }

pub async fn fetch_symbols_by_inst_type(inst_type: &str) -> Result<Vec<String>, String> {
    let url = format!("https://www.okx.com/api/v5/public/instruments?instType={inst_type}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("okx instruments status={}", resp.status()));
    }
    let body: Resp = resp.json().await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for i in body.data {
        if i.inst_type == inst_type { out.push(to_canonical_symbol(&i.inst_id)); }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// NEW: MarketMeta を返す（tick/step/min_qty/min_notional）
pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    Err("NotSupported: fetch_market_meta is not implemented for this connector yet".to_string())
}
