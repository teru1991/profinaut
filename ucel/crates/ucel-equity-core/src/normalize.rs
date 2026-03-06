use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use ucel_core::{EquityExchangeCode, EquityMarket, EquitySymbol};

pub fn normalize_exchange_code(raw: &str) -> Result<EquityExchangeCode, EquityAdapterError> {
    let code = raw.trim().to_ascii_uppercase();
    if code.is_empty() {
        return Err(EquityAdapterError::new(
            EquityAdapterErrorKind::MalformedResponse,
            "exchange code empty",
        ));
    }
    Ok(EquityExchangeCode(code))
}

pub fn normalize_symbol_key(
    market: &EquityMarket,
    exchange: &EquityExchangeCode,
    symbol: &str,
) -> String {
    let m = match market {
        EquityMarket::JP => "JP",
        EquityMarket::US => "US",
        EquityMarket::Other(v) => v,
    };
    format!("{}:{}:{}", m, exchange.0, symbol.to_ascii_uppercase())
}

pub fn ensure_unambiguous_symbols(items: &[EquitySymbol]) -> Result<(), EquityAdapterError> {
    let mut set = std::collections::HashSet::new();
    for s in items {
        let key = normalize_symbol_key(&s.market, &s.exchange, &s.canonical);
        if !set.insert(key) {
            return Err(EquityAdapterError::new(
                EquityAdapterErrorKind::AmbiguousSymbol,
                "duplicate canonical symbol mapping",
            ));
        }
    }
    Ok(())
}
