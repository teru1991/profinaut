use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Resp {
    #[serde(default)]
    result: ResultBody,
}
#[derive(Debug, Deserialize, Default)]
struct ResultBody {
    #[serde(default)]
    list: Vec<Item>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(default)]
    status: String,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    base_coin: Option<String>,
    #[serde(default)]
    quote_coin: Option<String>,
}

fn canon_pair(base: &str, quote: &str) -> String {
    format!("{base}/{quote}")
}

async fn fetch(category: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.bybit.com/v5/market/instruments-info?category={category}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bybit instruments-info status={}", resp.status()));
    }
    let body: Resp = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for it in body.result.list {
        if !it.status.eq_ignore_ascii_case("Trading") {
            continue;
        }
        if let Some(sym) = it.symbol {
            // optionsなどは raw symbol を優先（そのまま使う）
            out.push(sym);
        } else if let (Some(b), Some(q)) = (it.base_coin, it.quote_coin) {
            out.push(canon_pair(&b, &q));
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub async fn fetch_spot_symbols() -> Result<Vec<String>, String> {
    fetch("spot").await
}
pub async fn fetch_linear_symbols() -> Result<Vec<String>, String> {
    fetch("linear").await
}
pub async fn fetch_inverse_symbols() -> Result<Vec<String>, String> {
    fetch("inverse").await
}
pub async fn fetch_option_symbols() -> Result<Vec<String>, String> {
    fetch("option").await
}

pub fn to_exchange_symbol(symbol: &str) -> String {
    // canonical "BTC/USDT" -> "BTCUSDT", options/raw -> keep
    if symbol.contains('/') {
        symbol.replace('/', "")
    } else {
        symbol.to_string()
    }
}
