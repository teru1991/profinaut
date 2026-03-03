use std::{fs::OpenOptions, io::Write, path::Path, time::SystemTime};

use serde::Serialize;
use serde_json::Value;
use ucel_core::Decimal;
use ucel_symbol_core::{
    check_schema_compatibility, Exchange, InstrumentId, MarketType, Snapshot,
    StandardizedInstrument, SymbolStatus, SYMBOL_SCHEMA_VERSION,
};
use ucel_symbol_store::{StoreCheckpoint, SymbolStore};

#[derive(thiserror::Error, Debug)]
pub enum StoreBridgeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("snapshot parse: {0}")]
    SnapshotParse(String),
    #[error("schema incompatible: {0}")]
    SchemaIncompatible(u16),
}

#[derive(Serialize)]
struct CheckpointLogLine<'a> {
    ts_unix_ms: u64,
    exchange_id: &'a str,
    schema_version: u32,
    store_version: u64,
    digest_hex: String,
}

fn unix_ms(now: SystemTime) -> u64 {
    now.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn apply_snapshot_to_store(
    store: &SymbolStore,
    exchange_id: &str,
    snapshot_body: &Value,
) -> Result<StoreCheckpoint, StoreBridgeError> {
    let instruments = parse_standardized_instruments(snapshot_body)?;
    let snapshot = Snapshot::new_rest(
        instruments
            .into_iter()
            .filter(|i| i.id.exchange == exchange_from_id(exchange_id))
            .collect(),
    );
    let _events = store.apply_snapshot(snapshot);
    let cp = store.checkpoint();
    Ok(cp)
}

pub fn record_checkpoint_jsonl(
    path: &Path,
    exchange_id: &str,
    checkpoint: &StoreCheckpoint,
) -> Result<(), StoreBridgeError> {
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    let line = CheckpointLogLine {
        ts_unix_ms: unix_ms(SystemTime::now()),
        exchange_id,
        schema_version: checkpoint.schema_version.0,
        store_version: checkpoint.store_version,
        digest_hex: hex::encode(checkpoint.digest),
    };
    let s = serde_json::to_string(&line).expect("serialize");
    f.write_all(s.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

fn parse_standardized_instruments(
    v: &Value,
) -> Result<Vec<StandardizedInstrument>, StoreBridgeError> {
    let schema_u64 = v
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(SYMBOL_SCHEMA_VERSION as u64);
    let schema = u16::try_from(schema_u64)
        .map_err(|_| StoreBridgeError::SnapshotParse("schema_version overflow".into()))?;
    if !matches!(
        check_schema_compatibility(schema),
        ucel_symbol_core::SchemaCompatibility::Compatible
    ) {
        return Err(StoreBridgeError::SchemaIncompatible(schema));
    }

    let rows = v
        .get("instruments")
        .and_then(Value::as_array)
        .ok_or_else(|| StoreBridgeError::SnapshotParse("missing instruments[]".into()))?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let exchange = exchange_from_id(get_s(row, "exchange")?);
        let market_type = market_type_from_str(get_s(row, "market_type")?);
        let raw_symbol = get_s(row, "raw_symbol")?.to_string();
        let base = get_s(row, "base")?.to_string();
        let quote = get_s(row, "quote")?.to_string();
        let status = symbol_status_from_str(
            row.get("status")
                .and_then(Value::as_str)
                .unwrap_or("trading"),
        );
        let tick_size = parse_decimal(
            row.get("tick_size")
                .and_then(Value::as_str)
                .unwrap_or("0.0001"),
        )?;
        let lot_size = parse_decimal(
            row.get("lot_size")
                .and_then(Value::as_str)
                .unwrap_or("0.0001"),
        )?;
        let meta = row
            .get("meta")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();

        let id = InstrumentId {
            exchange: exchange.clone(),
            market_type: market_type.clone(),
            raw_symbol: raw_symbol.clone(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        };

        out.push(StandardizedInstrument {
            id,
            exchange,
            market_type,
            base,
            quote,
            raw_symbol,
            status,
            tick_size,
            lot_size,
            min_order_qty: None,
            max_order_qty: None,
            min_notional: None,
            price_precision: None,
            qty_precision: None,
            contract_size: None,
            meta,
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: schema,
        });
    }

    Ok(out)
}

fn get_s<'a>(row: &'a Value, key: &str) -> Result<&'a str, StoreBridgeError> {
    row.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| StoreBridgeError::SnapshotParse(format!("missing {key}")))
}

fn parse_decimal(s: &str) -> Result<Decimal, StoreBridgeError> {
    s.parse::<Decimal>()
        .map_err(|e| StoreBridgeError::SnapshotParse(format!("invalid decimal {s}: {e}")))
}

fn exchange_from_id(s: &str) -> Exchange {
    match s {
        "gmocoin" => Exchange::Gmocoin,
        "bitbank" => Exchange::Bitbank,
        "bitflyer" => Exchange::Bitflyer,
        "coincheck" => Exchange::Coincheck,
        other => Exchange::Other(other.to_string()),
    }
}

fn market_type_from_str(s: &str) -> MarketType {
    match s {
        "spot" => MarketType::Spot,
        "linear_perpetual" => MarketType::LinearPerpetual,
        "inverse_perpetual" => MarketType::InversePerpetual,
        "option" => MarketType::Option,
        "delivery" => MarketType::Delivery,
        "margin" => MarketType::Margin,
        other => MarketType::Other(other.to_string()),
    }
}

fn symbol_status_from_str(s: &str) -> SymbolStatus {
    match s {
        "trading" => SymbolStatus::Trading,
        "suspended" => SymbolStatus::Suspended,
        "maintenance" => SymbolStatus::Maintenance,
        other => SymbolStatus::Unknown(other.to_string()),
    }
}
