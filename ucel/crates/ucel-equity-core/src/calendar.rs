use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use ucel_core::{EquityMarketCalendar, EquitySessionKind};

pub fn calendar_has_timezone(c: &EquityMarketCalendar) -> bool {
    !c.timezone.trim().is_empty()
}

pub fn validate_sessions(c: &EquityMarketCalendar) -> Result<(), EquityAdapterError> {
    if !calendar_has_timezone(c) {
        return Err(EquityAdapterError::new(EquityAdapterErrorKind::CalendarUnavailable, "timezone missing"));
    }
    if c.sessions.is_empty() {
        return Err(EquityAdapterError::new(EquityAdapterErrorKind::CalendarUnavailable, "sessions missing"));
    }
    let has_regular = c.sessions.iter().any(|s| s.kind == EquitySessionKind::Regular);
    if !has_regular {
        return Err(EquityAdapterError::new(EquityAdapterErrorKind::CalendarUnavailable, "regular session missing"));
    }
    Ok(())
}
