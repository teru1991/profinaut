use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct AssetPairsResp {
    #[serde(default)]
    result: BTreeMap<String, PairInfo>,
}
#[derive(Debug, Deserialize)]
struct PairInfo {
    #[serde(default)]
    wsname: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<String>,
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let url = "https://api.kraken.com/0/public/AssetPairs";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("kraken AssetPairs status={}", resp.status()));
    }
    let body: AssetPairsResp = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for (_k, v) in body.result {
        if let Some(ws) = v.wsname {
            out.push(ws);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
