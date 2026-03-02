use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, time::SystemTime};
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketType, Snapshot, StandardizedInstrument, SymbolStatus,
    SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct DeribitResponse {
    result: Vec<InstrumentDto>,
}

#[derive(Debug, Deserialize)]
struct InstrumentDto {
    instrument_name: String,
    base_currency: String,
    quote_currency: String,
    kind: String,
    #[serde(default)]
    tick_size: Option<f64>,
    #[serde(default)]
    min_trade_amount: Option<f64>,
}

fn parse_decimal_f64(v: f64, field: &str, symbol: &str) -> Result<Decimal, String> {
    Decimal::from_str(&v.to_string())
        .map_err(|e| format!("deribit parse {field} symbol={symbol}: {e}"))
}

fn market_type(kind: &str, symbol: &str) -> MarketType {
    match kind {
        "spot" => MarketType::Spot,
        "option" => MarketType::Option,
        "future" if symbol.contains("PERPETUAL") => MarketType::LinearPerpetual,
        "future" => MarketType::Delivery,
        _ => MarketType::Other(kind.to_string()),
    }
}

fn map_instrument(s: InstrumentDto) -> Result<StandardizedInstrument, String> {
    let tick = parse_decimal_f64(
        s.tick_size
            .ok_or_else(|| format!("deribit missing tick_size symbol={}", s.instrument_name))?,
        "tick_size",
        &s.instrument_name,
    )?;
    let step = parse_decimal_f64(
        s.min_trade_amount.ok_or_else(|| {
            format!(
                "deribit missing min_trade_amount symbol={}",
                s.instrument_name
            )
        })?,
        "min_trade_amount",
        &s.instrument_name,
    )?;

    Ok(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Deribit,
            market_type: market_type(&s.kind, &s.instrument_name),
            raw_symbol: s.instrument_name.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Deribit,
        market_type: market_type(&s.kind, &s.instrument_name),
        base: s.base_currency,
        quote: s.quote_currency,
        raw_symbol: s.instrument_name,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: Some(step),
        max_order_qty: None,
        min_notional: None,
        price_precision: Some(tick.normalize().scale()),
        qty_precision: Some(step.normalize().scale()),
        contract_size: None,
        meta: BTreeMap::new(),
        ts_recv: SystemTime::now(),
        ts_event: None,
        schema_version: SYMBOL_SCHEMA_VERSION,
    })
}

fn parse_snapshot(chunks: Vec<DeribitResponse>) -> Result<Vec<StandardizedInstrument>, String> {
    let mut out = Vec::new();
    for chunk in chunks {
        for ins in chunk.result {
            out.push(map_instrument(ins)?);
        }
    }
    Ok(out)
}

pub fn fetch_symbols() -> Result<Vec<String>, String> {
    Err("NotSupported".into())
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let mut chunks = Vec::new();
    for currency in ["BTC", "ETH"] {
        for kind in ["spot", "option", "future"] {
            let url = format!("https://www.deribit.com/api/v2/public/get_instruments?currency={currency}&kind={kind}&expired=false");
            let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!(
                    "deribit get_instruments status={} currency={currency} kind={kind}",
                    resp.status()
                ));
            }
            let body: DeribitResponse = resp.json().await.map_err(|e| e.to_string())?;
            chunks.push(body);
        }
    }
    Ok(Snapshot::new_rest(parse_snapshot(chunks)?))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out = BTreeMap::new();
    for s in snapshot.instruments {
        let mut mm = MarketMeta::new(
            ucel_symbol_core::MarketMetaId::new(
                Exchange::Deribit,
                s.market_type.clone(),
                s.raw_symbol.clone(),
            ),
            s.tick_size,
            s.lot_size,
        );
        mm.min_qty = s.min_order_qty;
        out.insert(s.raw_symbol, mm);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_deribit_instruments() {
        let body: DeribitResponse = serde_json::from_str(r#"{"result":[{"instrument_name":"BTC-PERPETUAL","base_currency":"BTC","quote_currency":"USD","kind":"future","tick_size":0.5,"min_trade_amount":10.0}]}"#).unwrap();
        let instruments = parse_snapshot(vec![body]).unwrap();
        assert_eq!(instruments.len(), 1);
    }

    #[test]
    fn errors_without_tick_or_step() {
        let body: DeribitResponse = serde_json::from_str(r#"{"result":[{"instrument_name":"BTC-PERPETUAL","base_currency":"BTC","quote_currency":"USD","kind":"future","min_trade_amount":10.0}]}"#).unwrap();
        assert!(parse_snapshot(vec![body])
            .unwrap_err()
            .contains("missing tick_size"));
    }

    #[test]
    fn errors_without_step() {
        let body: DeribitResponse = serde_json::from_str(r#"{"result":[{"instrument_name":"BTC-PERPETUAL","base_currency":"BTC","quote_currency":"USD","kind":"future","tick_size":0.5}]}"#).unwrap();
        assert!(parse_snapshot(vec![body])
            .unwrap_err()
            .contains("missing min_trade_amount"));
    }
}
