use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use crate::DemoEquityAdapter;
use ucel_core::{EquityMarket, EquitySymbol};
use ucel_equity_core::normalize::{ensure_unambiguous_symbols, normalize_exchange_code};

pub fn list_symbols(adapter: &DemoEquityAdapter) -> Result<Vec<EquitySymbol>, EquityAdapterError> {
    let raw = adapter.http.get_json("/symbols")?;
    let arr = raw.as_array().ok_or_else(|| EquityAdapterError::new(EquityAdapterErrorKind::MalformedResponse, "symbols not array"))?;
    let mut out = Vec::new();
    for it in arr {
        let market = match it.get("market").and_then(|v| v.as_str()).unwrap_or("OTHER") {
            "JP" => EquityMarket::JP,
            "US" => EquityMarket::US,
            s => EquityMarket::Other(s.to_string()),
        };
        out.push(EquitySymbol {
            canonical: it.get("canonical").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            vendor_symbol: it.get("vendor").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            market,
            exchange: normalize_exchange_code(it.get("exchange").and_then(|v| v.as_str()).unwrap_or_default())?,
            timezone: it.get("timezone").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        });
    }
    ensure_unambiguous_symbols(&out)?;
    Ok(out)
}

pub fn resolve_symbol(adapter: &DemoEquityAdapter, raw: &str) -> Result<EquitySymbol, EquityAdapterError> {
    let symbols = list_symbols(adapter)?;
    let matches: Vec<_> = symbols
        .into_iter()
        .filter(|s| s.vendor_symbol.eq_ignore_ascii_case(raw) || s.canonical.eq_ignore_ascii_case(raw))
        .collect();
    if matches.is_empty() {
        return Err(EquityAdapterError::new(EquityAdapterErrorKind::UnsupportedSymbol, "symbol not found"));
    }
    if matches.len() > 1 {
        return Err(EquityAdapterError::new(EquityAdapterErrorKind::AmbiguousSymbol, "ambiguous symbol mapping"));
    }
    Ok(matches[0].clone())
}
