use serde::{Deserialize, Serialize};
use ucel_core::EquitySupport;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquityVendorSurfaceSupport {
    pub quotes: EquitySupport,
    pub bars_intraday: EquitySupport,
    pub bars_daily: EquitySupport,
    pub symbols: EquitySupport,
    pub calendar: EquitySupport,
    pub corporate_actions: EquitySupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquityVendorCapabilities {
    pub vendor_id: String,
    pub support: EquityVendorSurfaceSupport,
    pub realtime: bool,
    pub delayed: bool,
}
