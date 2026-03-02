use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::SystemTime;
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketMetaId, MarketType, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct Resp<T> {
    #[serde(default)]
    status: String,
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
struct Sym {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    state: String,

    #[serde(default, rename = "base-currency")]
    base_currency: String,
    #[serde(default, rename = "quote-currency")]
    quote_currency: String,

    #[serde(default, rename = "price-precision")]
    price_precision: u32,
    #[serde(default, rename = "amount-precision")]
    amount_precision: u32,

    #[serde(default, rename = "min-order-amt")]
    min_order_amt: Option<f64>,
    #[serde(default, rename = "min-order-value")]
    min_order_value: Option<f64>,
}

fn step_from_precision(p: u32) -> Decimal {
    Decimal::new(1, p)
}

fn d_f64(v: f64) -> Result<Decimal, String> {
    v.to_string().parse::<Decimal>().map_err(|e| e.to_string())
}

pub async fn fetch_symbol_snapshot() -> Result<Snapshot, String> {
    let url = "https://api.bittrade.co.jp/v1/common/symbols";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bittrade symbols status={}", resp.status()));
    }
    let body: Resp<Sym> = resp.json().await.map_err(|e| e.to_string())?;
    if body.status != "ok" && body.status != "OK" {
        return Err(format!("bittrade symbols api status={}", body.status));
    }

    let mut instruments = Vec::new();

    for s in body.data {
        if s.state != "online" && s.state != "ONLINE" {
            continue;
        }

        let tick = step_from_precision(s.price_precision);
        let step = step_from_precision(s.amount_precision);

        let min_qty = s.min_order_amt.map(d_f64).transpose()?;
        let min_notional = s.min_order_value.map(d_f64).transpose()?;

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Bittrade,
                market_type: MarketType::Spot,
                raw_symbol: s.symbol.clone(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Bittrade,
            market_type: MarketType::Spot,
            base: s.base_currency.to_uppercase(),
            quote: s.quote_currency.to_uppercase(),
            raw_symbol: s.symbol,
            status: SymbolStatus::Trading,
            tick_size: tick,
            lot_size: step,
            min_order_qty: min_qty,
            max_order_qty: None,
            min_notional,
            price_precision: Some(s.price_precision),
            qty_precision: Some(s.amount_precision),
            contract_size: None,
            meta: BTreeMap::new(),
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: SYMBOL_SCHEMA_VERSION,
        });
    }

    if instruments.is_empty() {
        return Err("bittrade: no instruments produced".into());
    }

    Ok(Snapshot::new_rest(instruments))
}

pub async fn fetch_all_symbols() -> Result<Vec<String>, String> {
    Ok(fetch_symbol_snapshot()
        .await?
        .instruments
        .into_iter()
        .map(|i| i.raw_symbol)
        .collect())
}

pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let snapshot = fetch_symbol_snapshot().await?;
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    for s in snapshot.instruments {
        let mut mm = MarketMeta::new(
            MarketMetaId::new(Exchange::Bittrade, MarketType::Spot, s.raw_symbol.clone()),
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
            .map_err(|e| format!("bittrade invalid meta symbol={} err={e}", s.raw_symbol))?;
        out.insert(s.raw_symbol, mm);
    }

    Ok(out)
}
