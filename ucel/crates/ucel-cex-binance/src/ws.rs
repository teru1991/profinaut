use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{fetch_all_symbols, to_exchange_symbol, to_ws_symbol};

#[derive(Debug, Clone)]
pub struct BinanceSpotWsAdapter;
impl BinanceSpotWsAdapter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BinanceSpotWsAdapter {
    fn default() -> Self {
        Self::new()
    }
}
fn topic_from_params(op_id: &str, symbol: &str, params: &Value) -> Result<String, String> {
    // SSOT path: planner_v2 writes the final topic into params["_topic"]
    if let Some(t) = params.get("_topic").and_then(|v| v.as_str()) {
        // If topic contains "{symbol}" already expanded by planner with canonical "BASE/QUOTE",
        // we must convert it into exchange ws symbol format if needed.
        // Our planner uses canonical symbol for {symbol}; for Binance streams it must be lowercase without "/".
        // We detect presence of '/' and convert the first token before '@'.
        if t.contains('/') {
            // "BTC/USDT@trade" -> "btcusdt@trade"
            let mut parts = t.split('@');
            let left = parts.next().unwrap_or("");
            let rest = parts.collect::<Vec<_>>().join("@");
            let raw = to_ws_symbol(&to_exchange_symbol(left));
            if rest.is_empty() {
                return Ok(raw);
            }
            return Ok(format!("{raw}@{rest}"));
        }
        return Ok(t.to_string());
    }

    // Fallback: old behavior (should not be used in v2)
    if symbol.is_empty() {
        return Err(format!("missing _topic for symbol-less op_id={op_id}"));
    }
    let raw = to_ws_symbol(&to_exchange_symbol(symbol));
    Ok(format!("{raw}@unknown"))
}

#[async_trait]
impl WsVenueAdapter for BinanceSpotWsAdapter {
    fn exchange_id(&self) -> &str {
        "binance-spot"
    }

    fn ws_url(&self) -> String {
        "wss://stream.binance.com:9443/ws".to_string()
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

        // combined wrapper: {"stream":"xxx","data":{...}}
        if let Some(_stream) = v.get("stream").and_then(|x| x.as_str()) {
            return InboundClass::Data {
                op_id: None,
                symbol: None,
                params_canon_hint: Some("{}".into()),
            };
        }

        // ack: {"result":null,"id":1}
        if v.get("result").is_some() && v.get("id").is_some() {
            return InboundClass::System;
        }

        InboundClass::System
    }
}
