use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, time::SystemTime};
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketType, Snapshot, StandardizedInstrument, SymbolStatus,
    SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstrumentDto {
    symbol: String,
    state: Option<String>,
    typ: Option<String>,
    root_symbol: Option<String>,
    quote_currency: Option<String>,
    tick_size: String,
    lot_size: String,
    #[serde(default)]
    min_order_qty: Option<String>,
    #[serde(default)]
    min_qty: Option<String>,
    #[serde(default)]
    min_notional: Option<String>,
}

fn parse_decimal(v: &str, field: &str, symbol: &str) -> Result<Decimal, String> {
    Decimal::from_str(v).map_err(|e| format!("bitmex parse {field} symbol={symbol}: {e}"))
}

fn market_type(typ: Option<&str>) -> MarketType {
    match typ {
        Some("FFWCSX") => MarketType::LinearPerpetual,
        Some("IFXXXP") => MarketType::InversePerpetual,
        _ => MarketType::Other("bitmex".to_string()),
    }
}

fn map_instrument(s: InstrumentDto) -> Result<Option<StandardizedInstrument>, String> {
    if !matches!(s.state.as_deref(), Some("Open") | Some("open")) {
        return Ok(None);
    }

    let tick = parse_decimal(&s.tick_size, "tick_size", &s.symbol)?;
    let step = parse_decimal(&s.lot_size, "lot_size", &s.symbol)?;
    let min_qty = s
        .min_order_qty
        .as_deref()
        .or(s.min_qty.as_deref())
        .map(|v| parse_decimal(v, "min_qty", &s.symbol))
        .transpose()?;
    let min_notional = s
        .min_notional
        .as_deref()
        .map(|v| parse_decimal(v, "min_notional", &s.symbol))
        .transpose()?;

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Bitmex,
            market_type: market_type(s.typ.as_deref()),
            raw_symbol: s.symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Bitmex,
        market_type: market_type(s.typ.as_deref()),
        base: s.root_symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
        quote: s.quote_currency.unwrap_or_else(|| "USD".to_string()),
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

fn parse_snapshot(body: Vec<InstrumentDto>) -> Result<Vec<StandardizedInstrument>, String> {
    body.into_iter()
        .map(map_instrument)
        .filter_map(Result::transpose)
        .collect()
}

pub fn fetch_symbols() -> Result<Vec<String>, String> {
    Err("NotSupported".into())
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = "https://www.bitmex.com/api/v1/instrument/active";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bitmex instrument/active status={}", resp.status()));
    }
    let body: Vec<InstrumentDto> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(Snapshot::new_rest(parse_snapshot(body)?))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out = BTreeMap::new();
    for s in snapshot.instruments {
        let mut mm = MarketMeta::new(
            ucel_symbol_core::MarketMetaId::new(
                Exchange::Bitmex,
                s.market_type.clone(),
                s.raw_symbol.clone(),
            ),
            s.tick_size,
            s.lot_size,
        );
        mm.min_qty = s.min_order_qty;
        mm.min_notional = s.min_notional;
        out.insert(s.raw_symbol, mm);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bitmex_active_instruments() {
        let body: Vec<InstrumentDto> = serde_json::from_str(r#"[{"symbol":"XBTUSD","state":"Open","tickSize":"0.5","lotSize":"1","minOrderQty":"1","rootSymbol":"XBT","quoteCurrency":"USD"}]"#).unwrap();
        let instruments = parse_snapshot(body).unwrap();
        assert_eq!(instruments.len(), 1);
    }

    #[test]
    fn errors_when_tick_missing() {
        let body: Vec<InstrumentDto> = serde_json::from_str(
            r#"[{"symbol":"XBTUSD","state":"Open","tickSize":"","lotSize":"1"}]"#,
        )
        .unwrap();
        let err = parse_snapshot(body).unwrap_err();
        assert!(err.contains("tick_size"));
    }

    #[test]
    fn errors_when_step_missing() {
        let body: Vec<InstrumentDto> = serde_json::from_str(
            r#"[{"symbol":"XBTUSD","state":"Open","tickSize":"0.5","lotSize":""}]"#,
        )
        .unwrap();
        let err = parse_snapshot(body).unwrap_err();
        assert!(err.contains("lot_size"));
    }
}
