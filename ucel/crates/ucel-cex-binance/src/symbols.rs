use serde::Deserialize;
use std::collections::BTreeMap;
use ucel_core::Decimal;
use ucel_symbol_core::MarketMeta;

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolInfo {
    symbol: String,
    status: String,
    base_asset: String,
    quote_asset: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    filters: Vec<Filter>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Filter {
    #[serde(rename = "filterType")]
    filter_type: String,
    #[serde(default)]
    tick_size: Option<String>,
    #[serde(default)]
    step_size: Option<String>,
    #[serde(default)]
    min_qty: Option<String>,
    #[serde(default)]
    min_notional: Option<String>,
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

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
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

    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();
    for s in body.symbols {
        if s.status != "TRADING" {
            continue;
        }
        let is_spot = s.permissions.is_empty() || s.permissions.iter().any(|p| p == "SPOT");
        if !is_spot {
            continue;
        }

        let mut tick: Option<Decimal> = None;
        let mut step: Option<Decimal> = None;
        let mut min_qty: Option<Decimal> = None;
        let mut min_notional: Option<Decimal> = None;

        for f in s.filters {
            match f.filter_type.as_str() {
                "PRICE_FILTER" => {
                    if let Some(v) = f.tick_size {
                        tick = Some(v.parse::<Decimal>().map_err(|e| e.to_string())?);
                    }
                }
                "LOT_SIZE" => {
                    if let Some(v) = f.step_size {
                        step = Some(v.parse::<Decimal>().map_err(|e| e.to_string())?);
                    }
                    if let Some(v) = f.min_qty {
                        min_qty = Some(v.parse::<Decimal>().map_err(|e| e.to_string())?);
                    }
                }
                "MIN_NOTIONAL" => {
                    if let Some(v) = f.min_notional {
                        min_notional = Some(v.parse::<Decimal>().map_err(|e| e.to_string())?);
                    }
                }
                _ => {}
            }
        }

        let tick =
            tick.ok_or_else(|| format!("binance spot missing tick_size symbol={}", s.symbol))?;
        let step =
            step.ok_or_else(|| format!("binance spot missing step_size symbol={}", s.symbol))?;

        let canonical = to_canonical_symbol(&s.base_asset, &s.quote_asset);
        let mm = MarketMeta {
            tick_size: tick,
            step_size: step,
            min_qty,
            min_notional,
            ..MarketMeta::new(
                ucel_symbol_core::MarketMetaId::new(
                    ucel_symbol_core::Exchange::Binance,
                    ucel_symbol_core::MarketType::Spot,
                    s.symbol,
                ),
                tick,
                step,
            )
        };
        mm.validate_basic()
            .map_err(|e| format!("binance spot invalid meta {canonical}: {e}"))?;
        out.insert(canonical, mm);
    }

    Ok(out)
}
