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
#[serde(rename_all = "camelCase")]
struct Resp {
    #[serde(default)]
    ret_code: i32,
    #[serde(default)]
    ret_msg: String,
    #[serde(default)]
    result: Option<ResultBody>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ResultBody {
    #[serde(default)]
    next_page_cursor: String,
    #[serde(default)]
    list: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(default)]
    status: String,
    #[serde(default)]
    symbol: String,

    #[serde(default)]
    base_coin: String,
    #[serde(default)]
    quote_coin: String,

    #[serde(default)]
    price_filter: Option<PriceFilter>,
    #[serde(default)]
    lot_size_filter: Option<LotSizeFilter>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceFilter {
    #[serde(default)]
    tick_size: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LotSizeFilter {
    // deriv/options
    #[serde(default)]
    qty_step: Option<String>,
    #[serde(default)]
    min_order_qty: Option<String>,
    #[serde(default)]
    min_notional_value: Option<String>,

    // spot variants
    #[serde(default)]
    base_precision: Option<String>,
    #[serde(default)]
    min_order_amt: Option<String>,
}

fn parse_decimal(field: &str, s: &str) -> Result<Decimal, String> {
    Decimal::from_str(s)
        .map_err(|e| format!("bybit: invalid decimal field={field} value={s} err={e}"))
}

fn precision_from_step(step: Decimal) -> u32 {
    step.normalize().scale()
}

fn market_type_from_category(category: &str) -> MarketType {
    match category {
        "spot" => MarketType::Spot,
        "linear" => MarketType::LinearPerpetual,
        "inverse" => MarketType::InversePerpetual,
        "option" => MarketType::Option,
        other => MarketType::Other(format!("bybit:{other}")),
    }
}

fn map_item(category: &str, it: Item) -> Result<Option<StandardizedInstrument>, String> {
    if !it.status.eq_ignore_ascii_case("Trading") {
        return Ok(None);
    }

    let pf = it
        .price_filter
        .as_ref()
        .ok_or_else(|| format!("bybit: missing priceFilter symbol={}", it.symbol))?;
    let tick = parse_decimal("priceFilter.tickSize", &pf.tick_size)?;

    let lf = it
        .lot_size_filter
        .as_ref()
        .ok_or_else(|| format!("bybit: missing lotSizeFilter symbol={}", it.symbol))?;

    // qty step:
    // - derivatives/options: qtyStep
    // - spot: basePrecision
    let step_str = lf
        .qty_step
        .as_deref()
        .or(lf.base_precision.as_deref())
        .ok_or_else(|| format!("bybit: missing qtyStep/basePrecision symbol={}", it.symbol))?;
    let step = parse_decimal("lotSizeFilter.qtyStep/basePrecision", step_str)?;

    if tick <= Decimal::ZERO || step <= Decimal::ZERO {
        return Err(format!(
            "bybit: non-positive tick/step symbol={} tick={tick} step={step}",
            it.symbol
        ));
    }

    let min_qty = lf
        .min_order_qty
        .as_deref()
        .map(|v| parse_decimal("lotSizeFilter.minOrderQty", v))
        .transpose()?;

    // min notional:
    // - derivatives: minNotionalValue
    // - spot: minOrderAmt (quote amount)
    let min_notional = lf
        .min_notional_value
        .as_deref()
        .or(lf.min_order_amt.as_deref())
        .map(|v| parse_decimal("lotSizeFilter.minNotionalValue/minOrderAmt", v))
        .transpose()?;

    let mt = market_type_from_category(category);

    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Bybit,
            market_type: mt.clone(),
            raw_symbol: it.symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Bybit,
        market_type: mt,
        base: it.base_coin,
        quote: it.quote_coin,
        raw_symbol: it.symbol,
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

pub async fn fetch_symbol_snapshot_by_category(category: &str) -> Result<Snapshot, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let mut cursor: Option<String> = None;
    let mut instruments: Vec<StandardizedInstrument> = Vec::new();
    let mut page_guard = 0usize;

    loop {
        page_guard += 1;
        if page_guard > 200 {
            return Err(format!(
                "bybit: pagination guard triggered category={category}"
            ));
        }

        let mut url = format!(
            "https://api.bybit.com/v5/market/instruments-info?category={category}&limit=1000"
        );
        if let Some(c) = cursor.as_deref() {
            if !c.is_empty() {
                url.push_str("&cursor=");
                url.push_str(c);
            }
        }

        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!(
                "bybit instruments-info status={} category={category}",
                resp.status()
            ));
        }

        let body: Resp = resp.json().await.map_err(|e| e.to_string())?;
        if body.ret_code != 0 {
            return Err(format!(
                "bybit instruments-info retCode={} msg={}",
                body.ret_code, body.ret_msg
            ));
        }

        let rb = body.result.unwrap_or_default();
        let next_cursor = rb.next_page_cursor.clone();

        for it in rb.list {
            match map_item(category, it) {
                Ok(Some(inst)) => instruments.push(inst),
                Ok(None) => {}
                Err(e) => return Err(e), // safety-first
            }
        }

        if next_cursor.is_empty() {
            break;
        }
        cursor = Some(next_cursor);
    }

    if instruments.is_empty() {
        return Err(format!(
            "bybit: no instruments produced category={category}"
        ));
    }

    Ok(Snapshot::new_rest(instruments))
}

async fn fetch_symbols(category: &str) -> Result<Vec<String>, String> {
    let snap = fetch_symbol_snapshot_by_category(category).await?;
    Ok(snap.instruments.into_iter().map(|i| i.raw_symbol).collect())
}

pub async fn fetch_spot_symbols() -> Result<Vec<String>, String> {
    fetch_symbols("spot").await
}
pub async fn fetch_linear_symbols() -> Result<Vec<String>, String> {
    fetch_symbols("linear").await
}
pub async fn fetch_inverse_symbols() -> Result<Vec<String>, String> {
    fetch_symbols("inverse").await
}
pub async fn fetch_option_symbols() -> Result<Vec<String>, String> {
    fetch_symbols("option").await
}

pub fn to_exchange_symbol(symbol: &str) -> String {
    // canonical "BTC/USDT" -> "BTCUSDT", options/raw -> keep
    if symbol.contains('/') {
        symbol.replace('/', "")
    } else {
        symbol.to_string()
    }
}

/// MarketMeta: 全カテゴリをmergeして返す（キーは raw_symbol）
pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    for category in ["spot", "linear", "inverse", "option"] {
        let snap = fetch_symbol_snapshot_by_category(category).await?;
        for s in snap.instruments {
            let mt = s.market_type.clone();
            let mut mm = MarketMeta::new(
                MarketMetaId::new(Exchange::Bybit, mt, s.raw_symbol.clone()),
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
                .map_err(|e| format!("bybit invalid meta symbol={} err={e}", s.raw_symbol))?;
            out.insert(s.raw_symbol, mm);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bybit_map_item_parses_tick_step_and_min() {
        let it: Item = serde_json::from_str(
            r#"{
              "status":"Trading",
              "symbol":"BTCUSDT",
              "baseCoin":"BTC",
              "quoteCoin":"USDT",
              "priceFilter":{"tickSize":"0.01"},
              "lotSizeFilter":{"qtyStep":"0.001","minOrderQty":"0.001","minNotionalValue":"10"}
            }"#,
        )
        .unwrap();

        let inst = map_item("linear", it).unwrap().unwrap();
        assert_eq!(inst.tick_size.to_string(), "0.01");
        assert_eq!(inst.lot_size.to_string(), "0.001");
        assert_eq!(inst.min_order_qty.unwrap().to_string(), "0.001");
        assert_eq!(inst.min_notional.unwrap().to_string(), "10");
    }

    #[test]
    fn bybit_missing_tick_or_step_is_error() {
        let it: Item = serde_json::from_str(
            r#"{
              "status":"Trading",
              "symbol":"BTCUSDT",
              "baseCoin":"BTC",
              "quoteCoin":"USDT",
              "lotSizeFilter":{"qtyStep":"0.001"}
            }"#,
        )
        .unwrap();
        let err = map_item("linear", it).unwrap_err();
        assert!(err.contains("missing priceFilter"));

        let it2: Item = serde_json::from_str(
            r#"{
              "status":"Trading",
              "symbol":"BTCUSDT",
              "baseCoin":"BTC",
              "quoteCoin":"USDT",
              "priceFilter":{"tickSize":"0.01"},
              "lotSizeFilter":{}
            }"#,
        )
        .unwrap();
        let err2 = map_item("linear", it2).unwrap_err();
        assert!(err2.contains("missing qtyStep/basePrecision"));
    }
}
