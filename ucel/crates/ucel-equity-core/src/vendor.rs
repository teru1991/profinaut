use crate::errors::EquityAdapterError;
use crate::models::EquityVendorCapabilities;
use ucel_core::{EquityBar, EquityCorporateAction, EquityMarketCalendar, EquityQuote, EquitySymbol};

pub trait EquityVendorAdapter {
    fn vendor_id(&self) -> &'static str;
    fn capabilities(&self) -> EquityVendorCapabilities;

    fn get_quote(&self, symbol: &str) -> Result<EquityQuote, EquityAdapterError>;
    fn get_bars(&self, symbol: &str, timeframe: &str, limit: usize) -> Result<Vec<EquityBar>, EquityAdapterError>;
    fn list_symbols(&self) -> Result<Vec<EquitySymbol>, EquityAdapterError>;
    fn get_market_calendar(&self, market: &str, date: &str) -> Result<EquityMarketCalendar, EquityAdapterError>;
    fn get_corporate_actions(&self, symbol: &str, from: &str, to: &str) -> Result<Vec<EquityCorporateAction>, EquityAdapterError>;
    fn resolve_symbol(&self, raw: &str) -> Result<EquitySymbol, EquityAdapterError>;
}
