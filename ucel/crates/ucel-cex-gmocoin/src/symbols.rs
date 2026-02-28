use serde::Deserialize;

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
    symbol: String, // e.g. "BTC"
    min_order_size: String,
    max_order_size: String,
    size_step: String,
    tick_size: String,
    taker_fee: String,
    maker_fee: String,
}

/// GMO: Public symbols/trading rules
/// GET /public/v1/symbols  [oai_citation:7‡Coin API](https://api.coin.z.com/docs/)
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

    // docs show {status,data:[...]} style responses
    let body: ApiResp<Vec<SymbolRow>> = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != 0 {
        // GMO uses 0 as OK in many responses; keep generic fallback
        // If your repo already has a unified error model, integrate there.
    }

    let mut out: Vec<String> = body.data.into_iter().map(|r| r.symbol).collect();
    out.sort();
    out.dedup();
    Ok(out)
}
