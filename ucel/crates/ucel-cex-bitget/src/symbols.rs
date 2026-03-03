use serde::Deserialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::SystemTime;
use ucel_core::Decimal;
use ucel_symbol_core::{
    Exchange, InstrumentId, MarketMeta, MarketMetaId, MarketType, Snapshot, StandardizedInstrument,
    SymbolStatus, SYMBOL_SCHEMA_VERSION,
};

/// Bitget REST base (main)
const REST_BASE: &str = "https://api.bitget.com";

#[derive(Debug, Deserialize)]
struct ApiResp<T> {
    code: String,
    msg: String,
    #[serde(default)]
    data: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpotSymbolRow {
    symbol: String, // e.g. BTCUSDT
    base_coin: String,
    quote_coin: String,
    #[serde(default)]
    min_trade_amount: String,
    #[serde(default)]
    min_trade_usdt: String,
    #[serde(default)]
    price_precision: String,
    #[serde(default)]
    quantity_precision: String,
    #[serde(default)]
    status: String, // online/gray/offline/halt
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FuturesContractRow {
    symbol: String, // e.g. BTCUSDT
    base_coin: String,
    quote_coin: String,

    #[serde(default)]
    min_trade_num: String,
    #[serde(default)]
    min_trade_usdt: String,

    #[serde(default)]
    price_end_step: String, // tick
    #[serde(default)]
    size_multiplier: String, // step

    #[serde(default, rename = "symbolStatus")]
    symbol_status: String, // normal/listed/maintain/...
}

fn parse_decimal(field: &str, s: &str) -> Result<Decimal, String> {
    Decimal::from_str(s)
        .map_err(|e| format!("bitget: invalid decimal field={field} value={s} err={e}"))
}

fn step_from_precision(p: u32) -> Decimal {
    // 10^-p
    Decimal::new(1, p)
}

fn precision_from_step(step: Decimal) -> u32 {
    step.normalize().scale()
}

/// SPOT snapshot
pub async fn fetch_spot_symbol_snapshot() -> Result<Snapshot, String> {
    let url = format!("{REST_BASE}/api/v2/spot/public/symbols");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("bitget spot symbols http status={}", resp.status()));
    }

    let body: ApiResp<SpotSymbolRow> = resp.json().await.map_err(|e| e.to_string())?;
    if body.code != "00000" {
        return Err(format!(
            "bitget spot symbols api code={} msg={}",
            body.code, body.msg
        ));
    }

    let mut instruments = Vec::new();

    for r in body.data {
        if r.status != "online" {
            continue;
        }

        let pp: u32 = r
            .price_precision
            .parse()
            .map_err(|_| format!("bitget: invalid pricePrecision={}", r.price_precision))?;
        let qp: u32 = r
            .quantity_precision
            .parse()
            .map_err(|_| format!("bitget: invalid quantityPrecision={}", r.quantity_precision))?;

        let tick = step_from_precision(pp);
        let step = step_from_precision(qp);

        let min_qty = if r.min_trade_amount.is_empty() {
            None
        } else {
            Some(parse_decimal("minTradeAmount", &r.min_trade_amount)?)
        };
        let min_notional = if r.min_trade_usdt.is_empty() {
            None
        } else {
            Some(parse_decimal("minTradeUSDT", &r.min_trade_usdt)?)
        };

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Bitget,
                market_type: MarketType::Spot,
                raw_symbol: r.symbol.clone(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Bitget,
            market_type: MarketType::Spot,
            base: r.base_coin,
            quote: r.quote_coin,
            raw_symbol: r.symbol,
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
        });
    }

    if instruments.is_empty() {
        return Err("bitget: no spot instruments produced".into());
    }

    Ok(Snapshot::new_rest(instruments))
}

/// Futures snapshot
pub async fn fetch_futures_symbol_snapshot(
    product_type: &str,
    market_type: MarketType,
) -> Result<Snapshot, String> {
    let url = format!("{REST_BASE}/api/v2/mix/market/contracts?productType={product_type}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "bitget futures contracts http status={}",
            resp.status()
        ));
    }

    let body: ApiResp<FuturesContractRow> = resp.json().await.map_err(|e| e.to_string())?;
    if body.code != "00000" {
        return Err(format!(
            "bitget futures contracts api code={} msg={}",
            body.code, body.msg
        ));
    }

    let mut instruments = Vec::new();

    for r in body.data {
        if r.symbol_status != "normal" {
            continue;
        }

        let tick = parse_decimal("priceEndStep", &r.price_end_step)?;
        let step = parse_decimal("sizeMultiplier", &r.size_multiplier)?;
        if tick <= Decimal::ZERO || step <= Decimal::ZERO {
            return Err(format!(
                "bitget: non-positive tick/step symbol={} tick={tick} step={step}",
                r.symbol
            ));
        }

        let min_qty = if r.min_trade_num.is_empty() {
            None
        } else {
            Some(parse_decimal("minTradeNum", &r.min_trade_num)?)
        };
        let min_notional = if r.min_trade_usdt.is_empty() {
            None
        } else {
            Some(parse_decimal("minTradeUSDT", &r.min_trade_usdt)?)
        };

        instruments.push(StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Bitget,
                market_type: market_type.clone(),
                raw_symbol: r.symbol.clone(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Bitget,
            market_type: market_type.clone(),
            base: r.base_coin,
            quote: r.quote_coin,
            raw_symbol: r.symbol,
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
        });
    }

    if instruments.is_empty() {
        return Err(format!(
            "bitget: no futures instruments produced product_type={product_type}"
        ));
    }

    Ok(Snapshot::new_rest(instruments))
}

/// Fetch all SPOT symbols (instId list like "BTCUSDT").
/// Endpoint: GET /api/v2/spot/public/symbols
pub async fn fetch_spot_symbols() -> Result<Vec<String>, String> {
    Ok(fetch_spot_symbol_snapshot()
        .await?
        .instruments
        .into_iter()
        .map(|i| i.raw_symbol)
        .collect())
}

/// Fetch futures contracts (instId list like "BTCUSDT") for productType:
/// "USDT-FUTURES" | "COIN-FUTURES" | "USDC-FUTURES"
/// Endpoint: GET /api/v2/mix/market/contracts?productType=...
pub async fn fetch_futures_symbols(product_type: &str) -> Result<Vec<String>, String> {
    // market_typeは “一覧” の用途では不要なので固定（callerが productType で判断）
    Ok(fetch_futures_symbol_snapshot(
        product_type,
        MarketType::Other(format!("bitget:{product_type}")),
    )
    .await?
    .instruments
    .into_iter()
    .map(|i| i.raw_symbol)
    .collect())
}

/// Convenience wrappers
pub async fn fetch_usdt_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("USDT-FUTURES").await
}
pub async fn fetch_coin_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("COIN-FUTURES").await
}
pub async fn fetch_usdc_futures_symbols() -> Result<Vec<String>, String> {
    fetch_futures_symbols("USDC-FUTURES").await
}

/// MarketMeta: spot + futures(3種) を merge（キーは raw_symbol）
pub async fn fetch_market_meta() -> Result<BTreeMap<String, MarketMeta>, String> {
    let mut out: BTreeMap<String, MarketMeta> = BTreeMap::new();

    // spot
    let snap = fetch_spot_symbol_snapshot().await?;
    for s in snap.instruments {
        let mut mm = MarketMeta::new(
            MarketMetaId::new(Exchange::Bitget, MarketType::Spot, s.raw_symbol.clone()),
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
            .map_err(|e| format!("bitget spot invalid meta symbol={} err={e}", s.raw_symbol))?;
        out.insert(s.raw_symbol, mm);
    }

    // futures: productTypeごとに market_type を固定して区別（将来のexecutor側分岐に使える）
    for (pt, mt) in [
        ("USDT-FUTURES", MarketType::LinearPerpetual),
        ("COIN-FUTURES", MarketType::InversePerpetual),
        (
            "USDC-FUTURES",
            MarketType::Other("bitget:usdc-futures".into()),
        ),
    ] {
        let snap = fetch_futures_symbol_snapshot(pt, mt.clone()).await?;
        for s in snap.instruments {
            let mut mm = MarketMeta::new(
                MarketMetaId::new(Exchange::Bitget, mt.clone(), s.raw_symbol.clone()),
                s.tick_size,
                s.lot_size,
            );
            mm.base = Some(s.base);
            mm.quote = Some(s.quote);
            mm.min_qty = s.min_order_qty;
            mm.min_notional = s.min_notional;
            mm.price_precision = s.price_precision;
            mm.qty_precision = s.qty_precision;
            mm.validate_basic().map_err(|e| {
                format!(
                    "bitget futures invalid meta symbol={} err={e}",
                    s.raw_symbol
                )
            })?;
            out.insert(s.raw_symbol, mm);
        }
    }

    Ok(out)
}

#[cfg(test)]
fn map_spot_row_for_test(r: &SpotSymbolRow) -> Result<Option<StandardizedInstrument>, String> {
    if r.status != "online" {
        return Ok(None);
    }
    let pp: u32 = r
        .price_precision
        .parse()
        .map_err(|_| format!("bitget invalid pricePrecision={}", r.price_precision))?;
    let qp: u32 = r
        .quantity_precision
        .parse()
        .map_err(|_| format!("bitget invalid quantityPrecision={}", r.quantity_precision))?;
    let tick = step_from_precision(pp);
    let step = step_from_precision(qp);
    let min_qty = if r.min_trade_amount.is_empty() {
        None
    } else {
        Some(parse_decimal("minTradeAmount", &r.min_trade_amount)?)
    };
    let min_notional = if r.min_trade_usdt.is_empty() {
        None
    } else {
        Some(parse_decimal("minTradeUSDT", &r.min_trade_usdt)?)
    };
    Ok(Some(StandardizedInstrument {
        id: InstrumentId {
            exchange: Exchange::Bitget,
            market_type: MarketType::Spot,
            raw_symbol: r.symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        exchange: Exchange::Bitget,
        market_type: MarketType::Spot,
        base: r.base_coin.clone(),
        quote: r.quote_coin.clone(),
        raw_symbol: r.symbol.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitget_spot_row_maps_precision_to_tick_step() {
        let r: SpotSymbolRow = serde_json::from_str(
            r#"{
              "symbol":"BTCUSDT",
              "baseCoin":"BTC",
              "quoteCoin":"USDT",
              "minTradeAmount":"0.001",
              "minTradeUsdt":"10",
              "pricePrecision":"2",
              "quantityPrecision":"3",
              "status":"online"
            }"#,
        )
        .unwrap();
        let inst = map_spot_row_for_test(&r).unwrap().unwrap();
        assert_eq!(inst.tick_size.to_string(), "0.01");
        assert_eq!(inst.lot_size.to_string(), "0.001");
        assert_eq!(inst.min_order_qty.unwrap().to_string(), "0.001");
        assert_eq!(inst.min_notional.unwrap().to_string(), "10");
    }

    #[test]
    fn bitget_invalid_precision_is_error() {
        let r: SpotSymbolRow = serde_json::from_str(
            r#"{
              "symbol":"BTCUSDT",
              "baseCoin":"BTC",
              "quoteCoin":"USDT",
              "pricePrecision":"x",
              "quantityPrecision":"3",
              "status":"online"
            }"#,
        )
        .unwrap();
        let err = map_spot_row_for_test(&r).unwrap_err();
        assert!(err.contains("invalid pricePrecision"));
    }
}
