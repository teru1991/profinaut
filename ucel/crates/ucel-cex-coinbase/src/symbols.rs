use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, time::SystemTime};
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketType, Snapshot, StandardizedInstrument, SymbolStatus,
    SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct ProductDto {
    id: String,
    base_currency: String,
    quote_currency: String,
    status: String,
    quote_increment: String,
    base_increment: String,
    #[serde(default)]
    min_market_funds: Option<String>,
    #[serde(default)]
    min_order_size: Option<String>,
    #[serde(default)]
    base_min_size: Option<String>,
    #[serde(default)]
    base_max_size: Option<String>,
}

fn parse_decimal(v: &str, field: &str, symbol: &str) -> Result<Decimal, String> {
    Decimal::from_str(v).map_err(|e| format!("coinbase parse {field} symbol={symbol}: {e}"))
}

fn map_product(p: ProductDto) -> Result<Option<StandardizedInstrument>, String> {
    if p.status != "online" {
        return Ok(None);
    }
    let tick = parse_decimal(&p.quote_increment, "quote_increment", &p.id)?;
    let step = parse_decimal(&p.base_increment, "base_increment", &p.id)?;
    let min_qty = p
        .base_min_size
        .as_deref()
        .or(p.min_order_size.as_deref())
        .map(|v| parse_decimal(v, "min_qty", &p.id))
        .transpose()?;
    let max_qty = p
        .base_max_size
        .as_deref()
        .map(|v| parse_decimal(v, "max_qty", &p.id))
        .transpose()?;
    let min_notional = p
        .min_market_funds
        .as_deref()
        .map(|v| parse_decimal(v, "min_notional", &p.id))
        .transpose()?;

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Coinbase,
            market_type: MarketType::Spot,
            raw_symbol: p.id.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Coinbase,
        market_type: MarketType::Spot,
        base: p.base_currency,
        quote: p.quote_currency,
        raw_symbol: p.id,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: min_qty,
        max_order_qty: max_qty,
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

fn parse_snapshot(body: Vec<ProductDto>) -> Result<Vec<StandardizedInstrument>, String> {
    body.into_iter()
        .map(map_product)
        .filter_map(Result::transpose)
        .collect()
}

pub fn fetch_symbols() -> Result<Vec<String>, String> {
    Err("NotSupported".into())
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = "https://api.exchange.coinbase.com/products";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("coinbase products status={}", resp.status()));
    }
    let body: Vec<ProductDto> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(Snapshot::new_rest(parse_snapshot(body)?))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out = BTreeMap::new();
    for s in snapshot.instruments {
        let canonical = format!("{}/{}", s.base, s.quote);
        let mut mm = MarketMeta::new(
            ucel_symbol_core::MarketMetaId::new(Exchange::Coinbase, MarketType::Spot, s.raw_symbol),
            s.tick_size,
            s.lot_size,
        );
        mm.min_qty = s.min_order_qty;
        mm.max_qty = s.max_order_qty;
        mm.min_notional = s.min_notional;
        out.insert(canonical, mm);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_coinbase_products() {
        let body: Vec<ProductDto> = serde_json::from_str(r#"[{"id":"BTC-USD","base_currency":"BTC","quote_currency":"USD","status":"online","quote_increment":"0.01","base_increment":"0.00000001","min_market_funds":"1"}]"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments.len(), 1);
    }

    #[test]
    fn fails_when_tick_missing() {
        let bad = serde_json::from_str::<Vec<ProductDto>>(
            r#"[{"id":"BTC-USD","base_currency":"BTC","quote_currency":"USD","status":"online","base_increment":"0.00000001"}]"#,
        );
        assert!(bad.is_err());
    }
}
