use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Resp {
    #[serde(default)]
    data: Vec<Item>,
}
#[derive(Debug, Deserialize)]
struct Item {
    #[serde(default)]
    #[allow(non_snake_case)]
    instId: String,
    #[serde(default)]
    #[allow(non_snake_case)]
    instType: String,
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
        if i.instType == inst_type { out.push(to_canonical_symbol(&i.instId)); }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
