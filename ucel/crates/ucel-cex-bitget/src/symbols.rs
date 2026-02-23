use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Resp<T> {
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct SpotSymbol {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    #[allow(non_snake_case)]
    baseCoin: Option<String>,
    #[serde(default)]
    #[allow(non_snake_case)]
    quoteCoin: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

fn canon_pair(base: &str, quote: &str) -> String {
    format!("{base}/{quote}")
}

pub fn to_exchange_symbol(symbol: &str) -> String {
    if symbol.contains('/') { symbol.replace('/', "") } else { symbol.to_string() }
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let url = "https://api.bitget.com/api/v2/spot/public/symbols";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bitget symbols status={}", resp.status()));
    }
    let body: Resp<SpotSymbol> = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for s in body.data {
        if let Some(st) = s.status.as_deref() {
            if st != "online" && st != "ONLINE" { continue; }
        }
        if let Some(sym) = s.symbol { out.push(sym); }
        else if let (Some(b), Some(q)) = (s.baseCoin, s.quoteCoin) { out.push(canon_pair(&b, &q)); }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
