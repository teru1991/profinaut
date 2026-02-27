use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::fetch_all_symbols;

#[derive(Debug, Clone)]
pub struct BinanceOptionsWsAdapter;
impl BinanceOptionsWsAdapter {
    pub fn new() -> Self {
        Self
    }
}

fn topic_from_params(_op_id: &str, _symbol: &str, params: &Value) -> Result<String, String> {
    if let Some(t) = params.get("_topic").and_then(|v| v.as_str()) {
        return Ok(t.to_string());
    }
    Err("missing _topic in params (planner_v2 required)".into())
}

#[async_trait]
impl WsVenueAdapter for BinanceOptionsWsAdapter {
    fn exchange_id(&self) -> &str {
        "binance-options"
    }

    fn ws_url(&self) -> String {
        // keep your existing base
        "wss://fstream.binance.com/public/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let topic = topic_from_params(op_id, symbol, params)?;
        Ok(vec![OutboundMsg {
            text: json!({"method":"SUBSCRIBE","params":[topic],"id":1}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };
        if v.get("stream").is_some() && v.get("data").is_some() {
            return InboundClass::Data {
                op_id: None,
                symbol: None,
                params_canon_hint: Some("{}".into()),
            };
        }
        if v.get("result").is_some() && v.get("id").is_some() {
            return InboundClass::System;
        }
        InboundClass::System
    }
}
