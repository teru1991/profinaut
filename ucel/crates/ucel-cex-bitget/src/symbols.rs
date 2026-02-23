use serde::Deserialize;

/// Bitget REST base (main)
const REST_BASE: &str = "https://api.bitget.com";

#[derive(Debug, Deserialize)]
struct ApiResp<T> {
    code: String,
    msg: String,
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct SpotSymbolRow {
    symbol: String,      // e.g. BTCUSDT
    #[serde(default)]
    status: String,      // online/gray/offline/halt
}

#[derive(Debug, Deserialize)]
struct FuturesContractRow {
    symbol: String,         // e.g. BTCUSDT
    #[serde(default)]
    symbolStatus: String,   // normal/listed/maintain/...
}

/// Fetch all SPOT symbols (instId list like "BTCUSDT").
/// Endpoint: GET /api/v2/spot/public/symbols :contentReference[oaicite:3]{index=3}
pub async fn fetch_spot_symbols() -> Result<Vec<String>, String> {
    let url = format!("{REST_BASE}/api/v2/spot/public/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bitget spot symbols http status={}", resp.status()));
    }

    let body: ApiResp<SpotSymbolRow> = resp.json().await.map_err(|e| e.to_string())?;
    if body.code != "00000" {
        return Err(format!("bitget spot symbols api code={} msg={}", body.code, body.msg));
    }

    let mut out = Vec::new();
    for r in body.data {
        // "online" is the normal tradable state. :contentReference[oaicite:4]{index=4}
        if r.status == "online" {
            out.push(r.symbol);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Fetch futures contracts (instId list like "BTCUSDT") for productType:
/// "USDT-FUTURES" | "COIN-FUTURES" | "USDC-FUTURES"
/// Endpoint: GET /api/v2/mix/market/contracts?productType=... :contentReference[oaicite:5]{index=5}
pub async fn fetch_futures_symbols(product_type: &str) -> Result<Vec<String>, String> {
    let url = format!("{REST_BASE}/api/v2/mix/market/contracts?productType={}", product_type);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bitget futures contracts http status={}", resp.status()));
    }

    let body: ApiResp<FuturesContractRow> = resp.json().await.map_err(|e| e.to_string())?;
    if body.code != "00000" {
        return Err(format!("bitget futures contracts api code={} msg={}", body.code, body.msg));
    }

    let mut out = Vec::new();
    for r in body.data {
        // "normal" is tradable. :contentReference[oaicite:6]{index=6}
        if r.symbolStatus == "normal" {
            out.push(r.symbol);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Convenience wrappers
pub async fn fetch_usdt_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("USDT-FUTURES").await
}
pub async fn fetch_coin_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("COIN-FUTURES").await
}
pub async fn fetch_usdc_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("USDC-FUTURES").await
}