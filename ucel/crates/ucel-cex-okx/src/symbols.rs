use serde::Deserialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::SystemTime;
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketMetaId, MarketType, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct Resp {
    #[serde(default)]
    data: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(default, rename = "instId")]
    inst_id: String,
    #[serde(default, rename = "instType")]
    inst_type: String,
    #[serde(default)]
    state: String,

    #[serde(default, rename = "baseCcy")]
    base_ccy: String,
    #[serde(default, rename = "quoteCcy")]
    quote_ccy: String,

    #[serde(default, rename = "tickSz")]
    tick_sz: String,
    #[serde(default, rename = "lotSz")]
    lot_sz: String,
    #[serde(default, rename = "minSz")]
    min_sz: String,
}

pub fn to_canonical_symbol(inst_id: &str) -> String {
    inst_id.replace('-', "/")
}
pub fn to_exchange_symbol(canonical: &str) -> String {
    canonical.replace('/', "-")
}

fn parse_decimal(field: &str, s: &str) -> Result<Decimal, String> {
    Decimal::from_str(s)
        .map_err(|e| format!("okx: invalid decimal field={field} value={s} err={e}"))
}

fn precision_from_step(step: Decimal) -> u32 {
    step.normalize().scale()
}

fn market_type_from_inst_type(inst_type: &str) -> MarketType {
    match inst_type {
        "SPOT" => MarketType::Spot,
        "SWAP" => MarketType::LinearPerpetual,
        "FUTURES" => MarketType::Delivery,
        "OPTION" => MarketType::Option,
        other => MarketType::Other(format!("okx:{other}")),
    }
}

pub async fn fetch_symbol_snapshot_by_inst_type(inst_type: &str) -> Result<Snapshot, String> {
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

    let mut instruments: Vec<StandardizedInstrument> = Vec::new();

    for i in body.data {
        if i.inst_type != inst_type {
            continue;
        }
        // tradable: "live"
        if i.state != "live" {
            continue;
        }

        let tick = parse_decimal("tickSz", &i.tick_sz)?;
        let step = parse_decimal("lotSz", &i.lot_sz)?;
        if tick <= Decimal::ZERO || step <= Decimal::ZERO {
            return Err(format!(
                "okx: non-positive tick/step instId={} tick={tick} step={step}",
                i.inst_id
            ));
        }

        let min_qty = if i.min_sz.is_empty() {
            None
        } else {
            Some(parse_decimal("minSz", &i.min_sz)?)
        };

        let mt = market_type_from_inst_type(&i.inst_type);

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Okx,
                market_type: mt.clone(),
                raw_symbol: i.inst_id.clone(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Okx,
            market_type: mt,
            base: i.base_ccy,
            quote: i.quote_ccy,
            raw_symbol: i.inst_id,
            status: SymbolStatus::Trading,
            tick_size: tick,
            lot_size: step,
            min_order_qty: min_qty,
            max_order_qty: None,
            min_notional: None,
            price_precision: Some(precision_from_step(tick)),
            qty_precision: Some(precision_from_step(step)),
            contract_size: None,
            meta: BTreeMap::new(),
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: SYMBOL_SCHEMA_VERSION,
        });
    }

    if instruments.is_empty() {
        return Err(format!("okx: no instruments produced instType={inst_type}"));
    }

    Ok(Snapshot::new_rest(instruments))
}

pub async fn fetch_symbols_by_inst_type(inst_type: &str) -> Result<Vec<String>, String> {
    let snap = fetch_symbol_snapshot_by_inst_type(inst_type).await?;
    let mut out = snap
        .instruments
        .into_iter()
        .map(|i| to_canonical_symbol(&i.raw_symbol))
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    Ok(out)
}

/// MarketMeta: 4 instType を merge（キーは instId=raw_symbol）
pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    for inst_type in ["SPOT", "SWAP", "FUTURES", "OPTION"] {
        let snap = fetch_symbol_snapshot_by_inst_type(inst_type).await?;
        for s in snap.instruments {
            let mt = s.market_type.clone();
            let mut mm = MarketMeta::new(
                MarketMetaId::new(Exchange::Okx, mt, s.raw_symbol.clone()),
                s.tick_size,
                s.lot_size,
            );
            mm.base = Some(s.base);
            mm.quote = Some(s.quote);
            mm.min_qty = s.min_order_qty;
            mm.price_precision = s.price_precision;
            mm.qty_precision = s.qty_precision;
            mm.validate_basic()
                .map_err(|e| format!("okx invalid meta symbol={} err={e}", s.raw_symbol))?;
            out.insert(s.raw_symbol, mm);
        }
    }

    Ok(out)
}

fn map_item_for_test(
    i: &Item,
    expected_inst_type: &str,
) -> Result<Option<StandardizedInstrument>, String> {
    if i.inst_type != expected_inst_type {
        return Ok(None);
    }
    if i.state != "live" {
        return Ok(None);
    }

    let tick = parse_decimal("tickSz", &i.tick_sz)?;
    let step = parse_decimal("lotSz", &i.lot_sz)?;
    if tick <= Decimal::ZERO || step <= Decimal::ZERO {
        return Err(format!(
            "okx: non-positive tick/step instId={} tick={tick} step={step}",
            i.inst_id
        ));
    }

    let min_qty = if i.min_sz.is_empty() {
        None
    } else {
        Some(parse_decimal("minSz", &i.min_sz)?)
    };
    let mt = market_type_from_inst_type(&i.inst_type);

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Okx,
            market_type: mt.clone(),
            raw_symbol: i.inst_id.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Okx,
        market_type: mt,
        base: i.base_ccy.clone(),
        quote: i.quote_ccy.clone(),
        raw_symbol: i.inst_id.clone(),
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: min_qty,
        max_order_qty: None,
        min_notional: None,
        price_precision: Some(precision_from_step(tick)),
        qty_precision: Some(precision_from_step(step)),
        contract_size: None,
        meta: BTreeMap::new(),
        ts_recv: SystemTime::now(),
        ts_event: None,
        schema_version: SYMBOL_SCHEMA_VERSION,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn okx_item_maps_tick_lot_min() {
        let i: Item = serde_json::from_str(
            r#"{
              "instId":"BTC-USDT",
              "instType":"SPOT",
              "state":"live",
              "baseCcy":"BTC",
              "quoteCcy":"USDT",
              "tickSz":"0.1",
              "lotSz":"0.001",
              "minSz":"0.001"
            }"#,
        )
        .unwrap();

        let inst = map_item_for_test(&i, "SPOT").unwrap().unwrap();
        assert_eq!(inst.tick_size.to_string(), "0.1");
        assert_eq!(inst.lot_size.to_string(), "0.001");
        assert_eq!(inst.min_order_qty.unwrap().to_string(), "0.001");
    }

    #[test]
    fn okx_missing_tick_or_step_is_error() {
        let i: Item = serde_json::from_str(
            r#"{
              "instId":"BTC-USDT",
              "instType":"SPOT",
              "state":"live",
              "baseCcy":"BTC",
              "quoteCcy":"USDT",
              "tickSz":"",
              "lotSz":"0.001",
              "minSz":""
            }"#,
        )
        .unwrap();

        let err = map_item_for_test(&i, "SPOT").unwrap_err();
        assert!(err.contains("invalid decimal"));
    }
}
