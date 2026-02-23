use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Resp<T> {
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Sym {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    state: String,
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let url = "https://api.bittrade.co.jp/v1/common/symbols";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bittrade symbols status={}", resp.status()));
    }
    let body: Resp<Sym> = resp.json().await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for s in body.data {
        if s.state == "online" || s.state == "ONLINE" {
            out.push(s.symbol);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
