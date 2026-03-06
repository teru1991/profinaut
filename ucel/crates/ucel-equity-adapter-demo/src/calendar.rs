use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use crate::DemoEquityAdapter;
use ucel_core::{
    EquityExchangeCode, EquityMarket, EquityMarketCalendar, EquitySessionKind, EquitySessionWindow,
};
use ucel_equity_core::calendar::validate_sessions;

pub fn get_market_calendar(adapter: &DemoEquityAdapter, market: &str, date: &str) -> Result<EquityMarketCalendar, EquityAdapterError> {
    let endpoint = format!("/calendar/{market}/{date}");
    let raw = adapter.http.get_json(&endpoint)?;
    let sessions = raw
        .get("sessions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| EquityAdapterError::new(EquityAdapterErrorKind::CalendarUnavailable, "sessions missing"))?;
    let mut ws = Vec::new();
    for s in sessions {
        let kind = match s.get("kind").and_then(|v| v.as_str()).unwrap_or("Closed") {
            "PreMarket" => EquitySessionKind::PreMarket,
            "Regular" => EquitySessionKind::Regular,
            "AfterHours" => EquitySessionKind::AfterHours,
            "Holiday" => EquitySessionKind::Holiday,
            _ => EquitySessionKind::Closed,
        };
        ws.push(EquitySessionWindow {
            kind,
            start_local: s.get("start").and_then(|v| v.as_str()).unwrap_or("00:00").to_string(),
            end_local: s.get("end").and_then(|v| v.as_str()).unwrap_or("00:00").to_string(),
        });
    }
    let out = EquityMarketCalendar {
        market: if market.eq_ignore_ascii_case("JP") { EquityMarket::JP } else { EquityMarket::US },
        exchange: EquityExchangeCode(raw.get("exchange").and_then(|v| v.as_str()).unwrap_or("UNKNOWN").to_string()),
        timezone: raw.get("timezone").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        date: date.to_string(),
        sessions: ws,
    };
    validate_sessions(&out)?;
    Ok(out)
}
