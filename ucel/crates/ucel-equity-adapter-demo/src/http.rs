use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};

#[derive(Debug, Clone)]
pub struct DemoHttpClient {
    pub vendor_id: String,
}

impl DemoHttpClient {
    pub fn get_json(&self, endpoint: &str) -> Result<serde_json::Value, EquityAdapterError> {
        match endpoint {
            "/quote/7203.T" => Ok(
                serde_json::json!({"symbol":"7203.T","bid":3500.0,"ask":3501.0,"last":3500.5,"ts_ms":1700000000000_u64,"latency":"delayed"}),
            ),
            "/bars/7203.T" => Ok(serde_json::json!([
                {"symbol":"7203.T","tf":"1d","open":3400.0,"high":3520.0,"low":3380.0,"close":3500.5,"volume":123456.0,"ts_open_ms":1699913600000_u64,"ts_close_ms":1700000000000_u64,"latency":"end_of_day"}
            ])),
            "/symbols" => Ok(serde_json::json!([
                {"canonical":"7203","vendor":"7203.T","market":"JP","exchange":"TSE","timezone":"Asia/Tokyo"},
                {"canonical":"AAPL","vendor":"AAPL","market":"US","exchange":"NASDAQ","timezone":"America/New_York"}
            ])),
            "/calendar/JP/2026-03-06" => Ok(
                serde_json::json!({"market":"JP","exchange":"TSE","timezone":"Asia/Tokyo","date":"2026-03-06","sessions":[{"kind":"Regular","start":"09:00","end":"15:00"}]}),
            ),
            "/actions/7203.T" => Ok(serde_json::json!([
                {"type":"dividend","ex_date":"2026-03-28","amount":45.0,"currency":"JPY"}
            ])),
            _ => Err(EquityAdapterError::new(
                EquityAdapterErrorKind::UnsupportedSymbol,
                "demo endpoint not found",
            )),
        }
    }
}
