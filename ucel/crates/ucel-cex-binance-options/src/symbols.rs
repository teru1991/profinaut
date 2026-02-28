use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    #[serde(default, rename = "optionSymbols")]
    option_symbols: Vec<OptionSymbol>,
}

#[derive(Debug, Deserialize)]
struct OptionSymbol {
    symbol: String,
    #[serde(default)]
    status: Option<String>,
}

pub fn to_ws_symbol(symbol: &str) -> String {
    // options symbols are already exchange-native; ws streams require lowercase in many contexts,
    // but options stream docs include e.g. ETH@trade (case-insensitive). We normalize to lowercase.
    symbol.to_lowercase()
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    // Options exchangeInfo: GET /eapi/v1/exchangeInfo (official)
    let url = "https://eapi.binance.com/eapi/v1/exchangeInfo";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "binance options exchangeInfo status={}",
            resp.status()
        ));
    }
    let body: ExchangeInfo = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for s in body.option_symbols {
        if let Some(st) = s.status.as_deref() {
            if st != "TRADING" && st != "trading" {
                continue;
            }
        }
        out.push(s.symbol);
    }
    out.sort();
    out.dedup();
    Ok(out)
}
