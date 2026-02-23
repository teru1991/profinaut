use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TickerResp {
    status: i32,
    data: Vec<TickerItem>,
}

#[derive(Debug, Deserialize)]
struct TickerItem {
    symbol: String,
}

fn to_canonical_symbol(s: &str) -> String {
    if s.contains('_') {
        s.replace('_', "/")
    } else {
        s.to_string()
    }
}

pub fn to_exchange_symbol(canonical: &str) -> String {
    if canonical.contains('/') {
        canonical.replace('/', "_")
    } else {
        canonical.to_string()
    }
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let url = "https://api.coin.z.com/public/v1/ticker";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("ticker http status={}", resp.status()));
    }

    let body: TickerResp = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != 0 {
        return Err(format!("ticker api status={}", body.status));
    }

    let mut out: Vec<String> = body
        .data
        .into_iter()
        .map(|x| to_canonical_symbol(&x.symbol))
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}
