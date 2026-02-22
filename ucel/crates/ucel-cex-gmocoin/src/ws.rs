use async_trait::async_trait;
use serde_json::{json, Value};

use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{fetch_all_symbols, to_exchange_symbol};

#[derive(Debug, Clone)]
pub struct GmoCoinWsAdapter;

impl GmoCoinWsAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WsVenueAdapter for GmoCoinWsAdapter {
    fn exchange_id(&self) -> &str {
        "gmocoin"
    }

    fn ws_url(&self) -> String {
        "wss://api.coin.z.com/ws/public/v1".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        _params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let channel = if op_id.contains("ticker") {
            "ticker"
        } else if op_id.contains("trade") || op_id.contains("trades") {
            "trades"
        } else if op_id.contains("orderbook") || op_id.contains("orderbooks") {
            "orderbooks"
        } else {
            return Err(format!("unsupported op_id={op_id}"));
        };

        let msg = json!({
            "command": "subscribe",
            "channel": channel,
            "symbol": to_exchange_symbol(symbol)
        });

        Ok(vec![OutboundMsg {
            text: msg.to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        let channel = v.get("channel").and_then(|x| x.as_str()).unwrap_or("");
        let symbol_raw = v
            .get("symbol")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        let symbol = symbol_raw.map(|s| {
            if s.contains('_') {
                s.replace('_', "/")
            } else {
                s
            }
        });

        let op_id = match channel {
            "ticker" => Some("crypto.public.ws.ticker".to_string()),
            "trades" => Some("crypto.public.ws.trades".to_string()),
            "orderbooks" => Some("crypto.public.ws.orderbooks".to_string()),
            _ => None,
        };

        if op_id.is_some() && symbol.is_some() {
            InboundClass::Data {
                op_id,
                symbol,
                params_canon_hint: Some("{}".to_string()),
            }
        } else {
            InboundClass::System
        }
    }
}
