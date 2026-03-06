use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use crate::symbols::resolve_symbol;
use crate::DemoEquityAdapter;
use ucel_core::{validate_bar_timeframe, EquityAdjustmentMode, EquityBar, EquityLatencyClass};

pub fn get_bars(adapter: &DemoEquityAdapter, symbol: &str, timeframe: &str, limit: usize) -> Result<Vec<EquityBar>, EquityAdapterError> {
    validate_bar_timeframe(timeframe).map_err(|e| EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, e.message))?;
    let resolved = resolve_symbol(adapter, symbol)?;
    let endpoint = format!("/bars/{}", resolved.vendor_symbol);
    let raw = adapter.http.get_json(&endpoint)?;
    let arr = raw.as_array().ok_or_else(|| EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "bars not array"))?;
    let mut out = Vec::new();
    for it in arr.iter().take(limit) {
        let latency = match it.get("latency").and_then(|v| v.as_str()).unwrap_or("delayed") {
            "realtime" => EquityLatencyClass::Realtime,
            "end_of_day" => EquityLatencyClass::EndOfDay,
            _ => EquityLatencyClass::Delayed,
        };
        out.push(EquityBar {
            symbol: resolved.clone(),
            timeframe: timeframe.to_string(),
            ts_open_ms: it.get("ts_open_ms").and_then(|v| v.as_u64()).unwrap_or(0),
            ts_close_ms: it.get("ts_close_ms").and_then(|v| v.as_u64()).unwrap_or(0),
            open: it.get("open").and_then(|v| v.as_f64()).unwrap_or(0.0),
            high: it.get("high").and_then(|v| v.as_f64()).unwrap_or(0.0),
            low: it.get("low").and_then(|v| v.as_f64()).unwrap_or(0.0),
            close: it.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0),
            volume: it.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.0),
            latency,
            adjustment_mode: EquityAdjustmentMode::SplitAdjusted,
        });
    }
    Ok(out)
}
