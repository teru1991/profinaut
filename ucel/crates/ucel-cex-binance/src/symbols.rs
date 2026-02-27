use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolInfo {
    status: String,
    base_asset: String,
    quote_asset: String,
    #[serde(default)]
    permissions: Vec<String>,
}

pub fn to_canonical_symbol(base: &str, quote: &str) -> String {
    format!("{base}/{quote}")
}

pub fn to_exchange_symbol(canonical: &str) -> String {
    canonical.replace('/', "")
}

pub fn to_ws_symbol(exchange_symbol: &str) -> String {
    exchange_symbol.to_lowercase()
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    // Spot exchangeInfo: GET /api/v3/exchangeInfo
    let url = "https://api.binance.com/api/v3/exchangeInfo";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "binance spot exchangeInfo status={}",
            resp.status()
        ));
    }
    let body: ExchangeInfo = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for s in body.symbols {
        if s.status != "TRADING" {
            continue;
        }
        // spot permissions guard (when available)
        let is_spot = s.permissions.is_empty() || s.permissions.iter().any(|p| p == "SPOT");
        if !is_spot {
            continue;
        }
        out.push(to_canonical_symbol(&s.base_asset, &s.quote_asset));
    }

    out.sort();
    out.dedup();
    Ok(out)
}
