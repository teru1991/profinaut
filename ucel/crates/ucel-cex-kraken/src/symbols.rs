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
struct AssetPairsResp {
    #[serde(default)]
    error: Vec<String>,
    #[serde(default)]
    result: BTreeMap<String, PairInfo>,
}

#[derive(Debug, Deserialize)]
struct PairInfo {
    #[serde(default)]
    wsname: Option<String>,
    #[serde(default)]
    status: Option<String>,

    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    quote: Option<String>,

    #[serde(default)]
    pair_decimals: Option<u32>,
    #[serde(default)]
    lot_decimals: Option<u32>,

    #[serde(default)]
    ordermin: Option<String>,
    #[serde(default)]
    costmin: Option<String>,
}

fn parse_decimal(field: &str, s: &str) -> Result<Decimal, String> {
    Decimal::from_str(s)
        .map_err(|e| format!("kraken: invalid decimal field={field} value={s} err={e}"))
}

fn step_from_precision(p: u32) -> Decimal {
    Decimal::new(1, p)
}

fn precision_from_step(step: Decimal) -> u32 {
    step.normalize().scale()
}

fn map_pair(key: String, v: PairInfo) -> Result<Option<StandardizedInstrument>, String> {
    if let Some(st) = v.status.as_deref() {
        if st != "online" && st != "Online" && st != "ONLINE" {
            return Ok(None);
        }
    }

    let raw_symbol = v.wsname.clone().unwrap_or(key);
    let pd = v.pair_decimals.unwrap_or(0);
    let ld = v.lot_decimals.unwrap_or(0);

    let tick = step_from_precision(pd);
    let step = step_from_precision(ld);

    let min_qty = v
        .ordermin
        .as_deref()
        .map(|s| parse_decimal("ordermin", s))
        .transpose()?;

    let min_notional = v
        .costmin
        .as_deref()
        .map(|s| parse_decimal("costmin", s))
        .transpose()?;

    let (base, quote) = if let (Some(b), Some(q)) = (v.base.clone(), v.quote.clone()) {
        (b, q)
    } else if raw_symbol.contains('/') {
        let mut sp = raw_symbol.split('/');
        (
            sp.next().unwrap_or("UNKNOWN").to_string(),
            sp.next().unwrap_or("UNKNOWN").to_string(),
        )
    } else {
        ("UNKNOWN".to_string(), "UNKNOWN".to_string())
    };

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Kraken,
            market_type: MarketType::Spot,
            raw_symbol: raw_symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Kraken,
        market_type: MarketType::Spot,
        base,
        quote,
        raw_symbol,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: min_qty,
        max_order_qty: None,
        min_notional,
        price_precision: Some(precision_from_step(tick)),
        qty_precision: Some(precision_from_step(step)),
        contract_size: None,
        meta: BTreeMap::new(),
        ts_recv: SystemTime::now(),
        ts_event: None,
        schema_version: SYMBOL_SCHEMA_VERSION,
    }))
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    let snap = fetch_symbol_snapshot().await?;
    let mut out = snap
        .instruments
        .into_iter()
        .map(|i| i.raw_symbol)
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    Ok(out)
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = "https://api.kraken.com/0/public/AssetPairs";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("kraken AssetPairs status={}", resp.status()));
    }
    let body: AssetPairsResp = resp.json().await.map_err(|e| e.to_string())?;
    if !body.error.is_empty() {
        return Err(format!("kraken AssetPairs error={:?}", body.error));
    }

    let mut instruments = Vec::new();
    for (k, v) in body.result {
        match map_pair(k, v) {
            Ok(Some(i)) => instruments.push(i),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    if instruments.is_empty() {
        return Err("kraken: no instruments produced".into());
    }

    Ok(Snapshot::new_rest(instruments))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    for s in snapshot.instruments {
        let mut mm = MarketMeta::new(
            MarketMetaId::new(Exchange::Kraken, MarketType::Spot, s.raw_symbol.clone()),
            s.tick_size,
            s.lot_size,
        );
        mm.base = Some(s.base);
        mm.quote = Some(s.quote);
        mm.min_qty = s.min_order_qty;
        mm.min_notional = s.min_notional;
        mm.price_precision = s.price_precision;
        mm.qty_precision = s.qty_precision;
        mm.validate_basic()
            .map_err(|e| format!("kraken invalid meta symbol={} err={e}", s.raw_symbol))?;
        out.insert(s.raw_symbol, mm);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_asset_pairs_fixture_without_network() {
        let body: AssetPairsResp = serde_json::from_str(
            r#"{
              "error": [],
              "result": {
                "XXBTZUSD": {
                  "wsname":"XBT/USD",
                  "status":"online",
                  "base":"XXBT",
                  "quote":"ZUSD",
                  "pair_decimals": 1,
                  "lot_decimals": 4,
                  "ordermin":"0.0001",
                  "costmin":"10"
                }
              }
            }"#,
        )
        .unwrap();

        let mut instruments = Vec::new();
        for (k, v) in body.result {
            instruments.push(map_pair(k, v).unwrap().unwrap());
        }
        assert_eq!(instruments.len(), 1);
        assert_eq!(instruments[0].tick_size.to_string(), "0.1");
        assert_eq!(instruments[0].lot_size.to_string(), "0.0001");
    }
}
