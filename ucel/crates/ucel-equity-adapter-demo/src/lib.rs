pub mod bars;
pub mod calendar;
pub mod corporate_actions;
pub mod errors;
pub mod http;
pub mod quote;
pub mod symbols;

use ucel_core::{
    EquityBar, EquityCorporateAction, EquityMarketCalendar, EquityQuote, EquitySupport,
    EquitySymbol,
};
use ucel_equity_core::errors::EquityAdapterError;
use ucel_equity_core::models::{EquityVendorCapabilities, EquityVendorSurfaceSupport};
use ucel_equity_core::vendor::EquityVendorAdapter;

#[derive(Debug, Clone)]
pub struct DemoEquityAdapter {
    pub http: http::DemoHttpClient,
}

impl Default for DemoEquityAdapter {
    fn default() -> Self {
        Self {
            http: http::DemoHttpClient {
                vendor_id: "demo-equity".into(),
            },
        }
    }
}

impl EquityVendorAdapter for DemoEquityAdapter {
    fn vendor_id(&self) -> &'static str {
        "demo-equity"
    }

    fn capabilities(&self) -> EquityVendorCapabilities {
        EquityVendorCapabilities {
            vendor_id: self.vendor_id().into(),
            support: EquityVendorSurfaceSupport {
                quotes: EquitySupport::Supported,
                bars_intraday: EquitySupport::Supported,
                bars_daily: EquitySupport::Supported,
                symbols: EquitySupport::Supported,
                calendar: EquitySupport::Supported,
                corporate_actions: EquitySupport::Supported,
            },
            realtime: false,
            delayed: true,
        }
    }

    fn get_quote(&self, symbol: &str) -> Result<EquityQuote, EquityAdapterError> {
        quote::get_quote(self, symbol)
    }

    fn get_bars(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: usize,
    ) -> Result<Vec<EquityBar>, EquityAdapterError> {
        bars::get_bars(self, symbol, timeframe, limit)
    }

    fn list_symbols(&self) -> Result<Vec<EquitySymbol>, EquityAdapterError> {
        symbols::list_symbols(self)
    }

    fn get_market_calendar(
        &self,
        market: &str,
        date: &str,
    ) -> Result<EquityMarketCalendar, EquityAdapterError> {
        calendar::get_market_calendar(self, market, date)
    }

    fn get_corporate_actions(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<EquityCorporateAction>, EquityAdapterError> {
        corporate_actions::get_corporate_actions(self, symbol, from, to)
    }

    fn resolve_symbol(&self, raw: &str) -> Result<EquitySymbol, EquityAdapterError> {
        symbols::resolve_symbol(self, raw)
    }
}
