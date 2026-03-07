use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use crate::symbols::resolve_symbol;
use crate::DemoEquityAdapter;
use ucel_core::{EquityAdjustmentMode, EquityLatencyClass, EquityQuote};

pub fn get_quote(
    adapter: &DemoEquityAdapter,
    symbol: &str,
) -> Result<EquityQuote, EquityAdapterError> {
    let resolved = resolve_symbol(adapter, symbol)?;
    let endpoint = format!("/quote/{}", resolved.vendor_symbol);
    let raw = adapter.http.get_json(&endpoint)?;
    let latency = match raw
        .get("latency")
        .and_then(|v| v.as_str())
        .unwrap_or("delayed")
    {
        "realtime" => EquityLatencyClass::Realtime,
        "end_of_day" => EquityLatencyClass::EndOfDay,
        _ => EquityLatencyClass::Delayed,
    };
    Ok(EquityQuote {
        symbol: resolved,
        bid: raw.get("bid").and_then(|v| v.as_f64()).ok_or_else(|| {
            EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "missing bid")
        })?,
        ask: raw.get("ask").and_then(|v| v.as_f64()).ok_or_else(|| {
            EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "missing ask")
        })?,
        last: raw.get("last").and_then(|v| v.as_f64()).ok_or_else(|| {
            EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "missing last")
        })?,
        ts_ms: raw.get("ts_ms").and_then(|v| v.as_u64()).ok_or_else(|| {
            EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "missing ts_ms")
        })?,
        latency,
        adjustment_mode: EquityAdjustmentMode::Raw,
    })
}
