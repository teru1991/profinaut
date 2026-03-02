use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, time::SystemTime};
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketType, OptionRight, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    #[serde(default, rename = "optionSymbols")]
    option_symbols: Vec<OptionSymbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionSymbol {
    symbol: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    underlying: Option<String>,
    #[serde(default)]
    quote_asset: Option<String>,
    #[serde(default)]
    expiry_date: Option<String>,
    #[serde(default)]
    strike_price: Option<String>,
    #[serde(default)]
    side: Option<String>,
    #[serde(default)]
    price_scale: Option<u32>,
    #[serde(default)]
    quantity_scale: Option<u32>,
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

fn parse_decimal(v: &str, field: &str, symbol: &str) -> Result<Decimal, String> {
    Decimal::from_str(v).map_err(|e| format!("binance options parse {field} symbol={symbol}: {e}"))
}

fn decimal_from_scale(scale: u32) -> Result<Decimal, String> {
    Ok(Decimal::new(1, scale))
}

fn parse_option_right(side: Option<&str>) -> Option<OptionRight> {
    match side {
        Some("CALL") | Some("C") => Some(OptionRight::Call),
        Some("PUT") | Some("P") => Some(OptionRight::Put),
        _ => None,
    }
}

fn map_symbol(s: OptionSymbol) -> Result<Option<StandardizedInstrument>, String> {
    if let Some(st) = s.status.as_deref() {
        if !st.eq_ignore_ascii_case("TRADING") {
            return Ok(None);
        }
    }

    let mut tick = None;
    let mut step = None;
    let mut min_qty = None;
    let mut min_notional = None;
    for f in s.filters {
        match f.filter_type.as_str() {
            "PRICE_FILTER" => {
                if let Some(v) = f.tick_size.as_deref() {
                    tick = Some(parse_decimal(v, "tick_size", &s.symbol)?);
                }
            }
            "LOT_SIZE" => {
                if let Some(v) = f.step_size.as_deref() {
                    step = Some(parse_decimal(v, "step_size", &s.symbol)?);
                }
                if let Some(v) = f.min_qty.as_deref() {
                    min_qty = Some(parse_decimal(v, "min_qty", &s.symbol)?);
                }
            }
            "MIN_NOTIONAL" => {
                if let Some(v) = f.min_notional.as_deref() {
                    min_notional = Some(parse_decimal(v, "min_notional", &s.symbol)?);
                }
            }
            _ => {}
        }
    }

    if tick.is_none() {
        if let Some(scale) = s.price_scale {
            tick = Some(decimal_from_scale(scale)?);
        }
    }
    if step.is_none() {
        if let Some(scale) = s.quantity_scale {
            step = Some(decimal_from_scale(scale)?);
        }
    }

    let tick =
        tick.ok_or_else(|| format!("binance options missing tick_size symbol={}", s.symbol))?;
    let step =
        step.ok_or_else(|| format!("binance options missing step_size symbol={}", s.symbol))?;

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::BinanceOptions,
            market_type: MarketType::Option,
            raw_symbol: s.symbol.clone(),
            expiry: s.expiry_date.clone(),
            strike: s
                .strike_price
                .as_deref()
                .map(|v| Decimal::from_str(v).map_err(|e| e.to_string()))
                .transpose()?,
            option_right: parse_option_right(s.side.as_deref()),
            contract_size: None,
        },
        exchange: Exchange::BinanceOptions,
        market_type: MarketType::Option,
        base: s.underlying.unwrap_or_else(|| "UNKNOWN".to_string()),
        quote: s.quote_asset.unwrap_or_else(|| "USDT".to_string()),
        raw_symbol: s.symbol,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: min_qty,
        max_order_qty: None,
        min_notional,
        price_precision: Some(tick.normalize().scale()),
        qty_precision: Some(step.normalize().scale()),
        contract_size: None,
        meta: BTreeMap::new(),
        ts_recv: SystemTime::now(),
        ts_event: None,
        schema_version: SYMBOL_SCHEMA_VERSION,
    }))
}

fn parse_snapshot(body: ExchangeInfo) -> Result<Vec<StandardizedInstrument>, String> {
    body.option_symbols
        .into_iter()
        .map(map_symbol)
        .filter_map(Result::transpose)
        .collect()
}

pub fn to_ws_symbol(symbol: &str) -> String {
    symbol.to_lowercase()
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out: Vec<String> = snapshot
        .instruments
        .into_iter()
        .map(|s| s.raw_symbol)
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
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
    Ok(Snapshot::new_rest(parse_snapshot(body)?))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out = BTreeMap::new();
    for s in snapshot.instruments {
        let mut mm = MarketMeta::new(
            ucel_symbol_core::MarketMetaId::new(
                Exchange::BinanceOptions,
                MarketType::Option,
                s.raw_symbol.clone(),
            ),
            s.tick_size,
            s.lot_size,
        );
        mm.min_qty = s.min_order_qty;
        mm.min_notional = s.min_notional;
        mm.price_precision = s.price_precision;
        mm.qty_precision = s.qty_precision;
        out.insert(s.raw_symbol, mm);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_option_with_filters() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"optionSymbols":[{"symbol":"BTC-240628-50000-C","status":"TRADING","underlying":"BTC","quoteAsset":"USDT","filters":[{"filterType":"PRICE_FILTER","tickSize":"0.1"},{"filterType":"LOT_SIZE","stepSize":"0.01"}]}]}"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments[0].tick_size.to_string(), "0.1");
    }

    #[test]
    fn errors_when_tick_cannot_be_extracted() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"optionSymbols":[{"symbol":"BTC-240628-50000-C","status":"TRADING","filters":[{"filterType":"LOT_SIZE","stepSize":"0.01"}]}]}"#).unwrap();
        assert!(parse_snapshot(body)
            .unwrap_err()
            .contains("missing tick_size"));
    }

    #[test]
    fn uses_scale_fallback_when_filters_missing() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"optionSymbols":[{"symbol":"BTC-240628-50000-C","status":"TRADING","priceScale":2,"quantityScale":3,"filters":[]}]}"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments[0].tick_size.to_string(), "0.01");
        assert_eq!(instruments[0].lot_size.to_string(), "0.001");
    }

    #[test]
    fn errors_when_no_filters_and_no_scales() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"optionSymbols":[{"symbol":"BTC-240628-50000-C","status":"TRADING","filters":[]}]}"#).unwrap();
        let err = parse_snapshot(body).unwrap_err();
        assert!(err.contains("missing tick_size"));
    }
}
