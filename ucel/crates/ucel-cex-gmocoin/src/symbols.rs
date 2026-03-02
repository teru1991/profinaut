use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::SystemTime;
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketMetaId, MarketType, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

const PUBLIC_BASE: &str = "https://api.coin.z.com/public";

#[derive(Debug, Deserialize)]
struct ApiResp<T> {
    status: u16,
    #[serde(default)]
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct SymbolRow {
    symbol: String,
    min_order_size: String,
    max_order_size: String,
    size_step: String,
    tick_size: String,
    taker_fee: String,
    maker_fee: String,
}

pub async fn fetch_symbols() -> Result<Vec<String>, String> {
    let url = format!("{PUBLIC_BASE}/v1/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("gmocoin symbols http status={}", resp.status()));
    }

    let body: ApiResp<Vec<SymbolRow>> = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != 0 {
        return Err(format!("gmocoin api status={}", body.status));
    }

    let mut out: Vec<String> = body.data.into_iter().map(|r| r.symbol).collect();
    out.sort();
    out.dedup();
    Ok(out)
}

/// NEW: Symbol Snapshot（Spotのみ）
/// - GMOの symbol は "BTC" のように base だけで返ることが多いので、quote=JPY として UCEL側で canonical 化（BASE/JPY）
pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = format!("{PUBLIC_BASE}/v1/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("gmocoin symbols http status={}", resp.status()));
    }

    let body: ApiResp<Vec<SymbolRow>> = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != 0 {
        return Err(format!("gmocoin api status={}", body.status));
    }

    let mut instruments = Vec::new();

    for r in body.data {
        let tick = r.tick_size.parse::<Decimal>().map_err(|e| e.to_string())?;
        let step = r.size_step.parse::<Decimal>().map_err(|e| e.to_string())?;
        let min_qty = r
            .min_order_size
            .parse::<Decimal>()
            .map_err(|e| e.to_string())?;

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Gmocoin,
                market_type: MarketType::Spot,
                raw_symbol: r.symbol.clone(), // raw is "BTC" etc
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Gmocoin,
            market_type: MarketType::Spot,
            base: r.symbol.clone(),
            quote: "JPY".to_string(),
            raw_symbol: r.symbol,
            status: SymbolStatus::Trading,
            tick_size: tick,
            lot_size: step,
            min_order_qty: Some(min_qty),
            max_order_qty: None,
            min_notional: None,
            price_precision: Some(tick.normalize().scale()),
            qty_precision: Some(step.normalize().scale()),
            contract_size: None,
            meta: BTreeMap::new(),
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: SYMBOL_SCHEMA_VERSION,
        });
    }

    if instruments.is_empty() {
        return Err("gmocoin: no instruments produced".into());
    }

    Ok(Snapshot::new_rest(instruments))
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    for s in snapshot.instruments {
        // canonical key: BASE/JPY
        let canonical = format!("{}/{}", s.base, s.quote);

        let mut mm = MarketMeta::new(
            MarketMetaId::new(Exchange::Gmocoin, MarketType::Spot, s.raw_symbol.clone()),
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
            .map_err(|e| format!("gmocoin invalid meta {canonical}: {e}"))?;
        out.insert(canonical, mm);
    }

    Ok(out)
}

fn map_row_for_test(r: SymbolRow) -> Result<StandardizedInstrument, String> {
    let tick = r.tick_size.parse::<Decimal>().map_err(|e| e.to_string())?;
    let step = r.size_step.parse::<Decimal>().map_err(|e| e.to_string())?;
    let min_qty = r
        .min_order_size
        .parse::<Decimal>()
        .map_err(|e| e.to_string())?;
    Ok(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Gmocoin,
            market_type: MarketType::Spot,
            raw_symbol: r.symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Gmocoin,
        market_type: MarketType::Spot,
        base: r.symbol.clone(),
        quote: "JPY".to_string(),
        raw_symbol: r.symbol,
        status: SymbolStatus::Trading,
        tick_size: tick,
        lot_size: step,
        min_order_qty: Some(min_qty),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmocoin_row_maps_tick_step_min() {
        let r: SymbolRow = serde_json::from_str(
            r#"{"symbol":"BTC","minOrderSize":"0.01","maxOrderSize":"100","sizeStep":"0.001","tickSize":"0.01","takerFee":"0","makerFee":"0"}"#,
        )
        .unwrap();
        let i = map_row_for_test(r).unwrap();
        assert_eq!(i.tick_size.to_string(), "0.01");
        assert_eq!(i.lot_size.to_string(), "0.001");
        assert_eq!(i.min_order_qty.unwrap().to_string(), "0.01");
    }

    #[test]
    fn gmocoin_invalid_tick_is_error() {
        let r: SymbolRow = serde_json::from_str(
            r#"{"symbol":"BTC","minOrderSize":"0.01","maxOrderSize":"100","sizeStep":"0.001","tickSize":"","takerFee":"0","makerFee":"0"}"#,
        )
        .unwrap();
        assert!(map_row_for_test(r).is_err());
    }
}
