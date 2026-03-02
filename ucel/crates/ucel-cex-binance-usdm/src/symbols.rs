use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, time::SystemTime};
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketType, Snapshot, StandardizedInstrument, SymbolStatus,
    SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolInfo {
    symbol: String,
    base_asset: String,
    quote_asset: String,
    status: String,
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
    #[serde(default)]
    notional: Option<String>,
}

fn parse_decimal(v: &str, field: &str, symbol: &str) -> Result<Decimal, String> {
    Decimal::from_str(v).map_err(|e| format!("binance usdm parse {field} symbol={symbol}: {e}"))
}

fn precision_from_decimal(value: Decimal) -> Option<u32> {
    Some(value.normalize().scale())
}

fn map_symbol(s: SymbolInfo) -> Result<Option<StandardizedInstrument>, String> {
    if s.status != "TRADING" {
        return Ok(None);
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
                if let Some(v) = f.min_notional.as_deref().or(f.notional.as_deref()) {
                    min_notional = Some(parse_decimal(v, "min_notional", &s.symbol)?);
                }
            }
            _ => {}
        }
    }

    let tick = tick.ok_or_else(|| format!("binance usdm missing tick_size symbol={}", s.symbol))?;
    let step = step.ok_or_else(|| format!("binance usdm missing step_size symbol={}", s.symbol))?;

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::BinanceUsdm,
            market_type: MarketType::LinearPerpetual,
            raw_symbol: s.symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::BinanceUsdm,
        market_type: MarketType::LinearPerpetual,
        base: s.base_asset,
        quote: s.quote_asset,
        raw_symbol: s.symbol,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: min_qty,
        max_order_qty: None,
        min_notional,
        price_precision: precision_from_decimal(tick),
        qty_precision: precision_from_decimal(step),
        contract_size: None,
        meta: BTreeMap::new(),
        ts_recv: SystemTime::now(),
        ts_event: None,
        schema_version: SYMBOL_SCHEMA_VERSION,
    }))
}

fn parse_snapshot(body: ExchangeInfo) -> Result<Vec<StandardizedInstrument>, String> {
    body.symbols
        .into_iter()
        .map(map_symbol)
        .filter_map(Result::transpose)
        .collect()
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
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out: Vec<String> = snapshot
        .instruments
        .into_iter()
        .map(|s| to_canonical_symbol(&s.base, &s.quote))
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = "https://fapi.binance.com/fapi/v1/exchangeInfo";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "binance usdm exchangeInfo status={}",
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
        let canonical = to_canonical_symbol(&s.base, &s.quote);
        let mut mm = MarketMeta::new(
            ucel_symbol_core::MarketMetaId::new(
                Exchange::BinanceUsdm,
                MarketType::LinearPerpetual,
                s.raw_symbol,
            ),
            s.tick_size,
            s.lot_size,
        );
        mm.min_qty = s.min_order_qty;
        mm.min_notional = s.min_notional;
        mm.price_precision = s.price_precision;
        mm.qty_precision = s.qty_precision;
        mm.validate_basic()
            .map_err(|e| format!("binance usdm invalid meta {canonical}: {e}"))?;
        out.insert(canonical, mm);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_maps_usdm_symbol() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"symbols":[{"symbol":"BTCUSDT","baseAsset":"BTC","quoteAsset":"USDT","status":"TRADING","filters":[{"filterType":"PRICE_FILTER","tickSize":"0.1"},{"filterType":"LOT_SIZE","stepSize":"0.001","minQty":"0.001"},{"filterType":"MIN_NOTIONAL","notional":"5"}]}]}"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments[0].tick_size.to_string(), "0.1");
        assert_eq!(instruments[0].lot_size.to_string(), "0.001");
    }

    #[test]
    fn errors_without_step() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"symbols":[{"symbol":"BTCUSDT","baseAsset":"BTC","quoteAsset":"USDT","status":"TRADING","filters":[{"filterType":"PRICE_FILTER","tickSize":"0.1"}]}]}"#).unwrap();
        let err = parse_snapshot(body).unwrap_err();
        assert!(err.contains("missing step_size"));
    }

    #[test]
    fn parses_min_notional_aliases() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"symbols":[{"symbol":"BTCUSDT","baseAsset":"BTC","quoteAsset":"USDT","status":"TRADING","filters":[{"filterType":"PRICE_FILTER","tickSize":"0.1"},{"filterType":"LOT_SIZE","stepSize":"0.001","minQty":"0.001"},{"filterType":"MIN_NOTIONAL","minNotional":"7"}]}]}"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments[0].min_notional.unwrap().to_string(), "7");
    }

    #[test]
    fn errors_without_tick() {
        let body: ExchangeInfo = serde_json::from_str(r#"{"symbols":[{"symbol":"BTCUSDT","baseAsset":"BTC","quoteAsset":"USDT","status":"TRADING","filters":[{"filterType":"LOT_SIZE","stepSize":"0.001"}]}]}"#).unwrap();
        let err = parse_snapshot(body).unwrap_err();
        assert!(err.contains("missing tick_size"));
    }
}
