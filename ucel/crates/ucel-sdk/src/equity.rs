use ucel_core::{EquityBar, EquityCorporateAction, EquityMarketCalendar, EquityQuote, EquitySymbol};
use ucel_equity_adapter_demo::DemoEquityAdapter;
use ucel_equity_core::errors::EquityAdapterError;
use ucel_equity_core::vendor::EquityVendorAdapter;

pub struct EquityDataFacade {
    adapter: Box<dyn EquityVendorAdapter + Send + Sync>,
}

impl Default for EquityDataFacade {
    fn default() -> Self {
        Self { adapter: Box::new(DemoEquityAdapter::default()) }
    }
}

impl EquityDataFacade {
    pub fn get_quote(&self, symbol: &str) -> Result<EquityQuote, EquityAdapterError> {
        self.adapter.get_quote(symbol)
    }

    pub fn get_bars(&self, symbol: &str, timeframe: &str, limit: usize) -> Result<Vec<EquityBar>, EquityAdapterError> {
        self.adapter.get_bars(symbol, timeframe, limit)
    }

    pub fn list_symbols(&self) -> Result<Vec<EquitySymbol>, EquityAdapterError> {
        self.adapter.list_symbols()
    }

    pub fn get_market_calendar(&self, market: &str, date: &str) -> Result<EquityMarketCalendar, EquityAdapterError> {
        self.adapter.get_market_calendar(market, date)
    }

    pub fn get_corporate_actions(&self, symbol: &str, from: &str, to: &str) -> Result<Vec<EquityCorporateAction>, EquityAdapterError> {
        self.adapter.get_corporate_actions(symbol, from, to)
    }

    pub fn resolve_symbol(&self, raw: &str) -> Result<EquitySymbol, EquityAdapterError> {
        self.adapter.resolve_symbol(raw)
    }

    pub fn preview_equity_vendor_plan(&self) -> serde_json::Value {
        let cap = self.adapter.capabilities();
        serde_json::json!({
            "vendor": cap.vendor_id,
            "realtime": cap.realtime,
            "delayed": cap.delayed,
            "supports": {
                "quotes": format!("{:?}", cap.support.quotes),
                "bars_intraday": format!("{:?}", cap.support.bars_intraday),
                "bars_daily": format!("{:?}", cap.support.bars_daily),
                "symbols": format!("{:?}", cap.support.symbols),
                "calendar": format!("{:?}", cap.support.calendar),
                "corporate_actions": format!("{:?}", cap.support.corporate_actions)
            }
        })
    }
}
