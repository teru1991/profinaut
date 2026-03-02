use serde::Deserialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::SystemTime;
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketMetaId, MarketType, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

include!(concat!(env!("OUT_DIR"), "/embedded_catalog.rs"));

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub schema_version: u16,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entry {
    pub exchange: String,
    pub market_type: String,
    pub raw_symbol: String,
    pub base: Option<String>,
    pub quote: Option<String>,
    pub tick_size: String,
    pub step_size: String,
    pub min_qty: Option<String>,
    pub min_notional: Option<String>,
    pub price_precision: Option<u32>,
    pub qty_precision: Option<u32>,
    pub note: Option<String>,
}

fn parse_decimal(field: &str, s: &str) -> Result<Decimal, String> {
    Decimal::from_str(s)
        .map_err(|e| format!("catalog invalid decimal field={field} value={s} err={e}"))
}

fn map_exchange(s: &str) -> Option<Exchange> {
    match s {
        "bitbank" => Some(Exchange::Bitbank),
        "bitflyer" => Some(Exchange::Bitflyer),
        "coincheck" => Some(Exchange::Coincheck),
        "sbivc" => Some(Exchange::Sbivc),
        "upbit" => Some(Exchange::Upbit),
        _ => None,
    }
}

fn map_market_type(s: &str) -> Option<MarketType> {
    match s {
        "spot" => Some(MarketType::Spot),
        "linear_perp" => Some(MarketType::LinearPerpetual),
        "inverse_perp" => Some(MarketType::InversePerpetual),
        "delivery" => Some(MarketType::Delivery),
        "option" => Some(MarketType::Option),
        _ => None,
    }
}

fn load() -> Catalog {
    serde_json::from_str(EMBEDDED_CATALOG_JSON).expect("embedded catalog json must be valid")
}

pub fn get_meta(
    exchange: Exchange,
    market_type: MarketType,
    raw_symbol: &str,
) -> Option<MarketMeta> {
    let cat = load();
    for e in cat.entries {
        let ex = map_exchange(&e.exchange)?;
        let mt = map_market_type(&e.market_type)?;
        if ex == exchange && mt == market_type && e.raw_symbol == raw_symbol {
            let tick = parse_decimal("tick_size", &e.tick_size).ok()?;
            let step = parse_decimal("step_size", &e.step_size).ok()?;
            if tick <= Decimal::ZERO || step <= Decimal::ZERO {
                return None;
            }

            let mut mm = MarketMeta::new(
                MarketMetaId::new(exchange, market_type, raw_symbol.to_string()),
                tick,
                step,
            );
            mm.base = e.base.clone();
            mm.quote = e.quote.clone();
            mm.min_qty = e
                .min_qty
                .as_deref()
                .and_then(|v| parse_decimal("min_qty", v).ok());
            mm.min_notional = e
                .min_notional
                .as_deref()
                .and_then(|v| parse_decimal("min_notional", v).ok());
            mm.price_precision = e.price_precision;
            mm.qty_precision = e.qty_precision;
            if let Some(note) = e.note {
                mm.meta
                    .insert("catalog_note".to_string(), serde_json::json!(note));
            }

            mm.validate_basic().ok()?;
            return Some(mm);
        }
    }
    None
}

pub fn snapshot_for_exchange(exchange: Exchange) -> Snapshot {
    let cat = load();
    let mut instruments: Vec<StandardizedInstrument> = Vec::new();

    for e in cat.entries {
        let ex = match map_exchange(&e.exchange) {
            Some(v) => v,
            None => continue,
        };
        if ex != exchange {
            continue;
        }
        let mt = match map_market_type(&e.market_type) {
            Some(v) => v,
            None => continue,
        };

        let tick = match parse_decimal("tick_size", &e.tick_size) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let step = match parse_decimal("step_size", &e.step_size) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if tick <= Decimal::ZERO || step <= Decimal::ZERO {
            continue;
        }

        let base = e.base.clone().unwrap_or_else(|| "UNKNOWN".to_string());
        let quote = e.quote.clone().unwrap_or_else(|| "UNKNOWN".to_string());
        let mut meta = BTreeMap::new();
        if let Some(note) = e.note.clone() {
            meta.insert("catalog_note".to_string(), serde_json::json!(note));
        }

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: exchange.clone(),
                market_type: mt.clone(),
                raw_symbol: e.raw_symbol.clone(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: exchange.clone(),
            market_type: mt,
            base,
            quote,
            raw_symbol: e.raw_symbol,
            status: SymbolStatus::Trading,
            tick_size: tick,
            lot_size: step,
            min_order_qty: e
                .min_qty
                .as_deref()
                .and_then(|v| parse_decimal("min_qty", v).ok()),
            max_order_qty: None,
            min_notional: e
                .min_notional
                .as_deref()
                .and_then(|v| parse_decimal("min_notional", v).ok()),
            price_precision: e.price_precision,
            qty_precision: e.qty_precision,
            contract_size: None,
            meta,
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: SYMBOL_SCHEMA_VERSION,
        });
    }

    Snapshot::new_rest(instruments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_parses() {
        let c = load();
        assert!(c.schema_version >= 1);
        assert!(
            !c.entries.is_empty(),
            "catalog must have at least one entry for tests"
        );
    }

    #[test]
    fn get_meta_returns_some_for_existing_entry() {
        let c = load();
        let e = c.entries[0].clone();
        let ex = map_exchange(&e.exchange).unwrap();
        let mt = map_market_type(&e.market_type).unwrap();
        let mm = get_meta(ex, mt, &e.raw_symbol).expect("meta must exist");
        assert!(mm.validate_basic().is_ok());
    }
}
