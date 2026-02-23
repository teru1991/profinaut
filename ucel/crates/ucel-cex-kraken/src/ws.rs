use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::fetch_all_symbols;

#[derive(Debug, Clone)]
pub struct KrakenSpotWsAdapter;
impl KrakenSpotWsAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WsVenueAdapter for KrakenSpotWsAdapter {
    fn exchange_id(&self) -> &str {
        "kraken"
    }
    fn ws_url(&self) -> String {
        "wss://ws.kraken.com/".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn ping_msg(&self) -> Option<OutboundMsg> {
        Some(OutboundMsg {
            text: json!({"event":"ping"}).to_string(),
        })
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let msg = match op_id {
            "kraken.public.ws.ticker" => {
                json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"ticker"}})
            }
            "kraken.public.ws.trade" => {
                json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"trade"}})
            }
            "kraken.public.ws.book" => {
                let depth = params.get("depth").and_then(|v| v.as_u64()).unwrap_or(10);
                json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"book","depth":depth}})
            }
            "kraken.public.ws.ohlc" => {
                let interval = params.get("interval").and_then(|v| v.as_u64()).unwrap_or(1);
                json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"ohlc","interval":interval}})
            }
            "kraken.public.ws.spread" => {
                json!({"event":"subscribe","pair":[symbol],"subscription":{"name":"spread"}})
            }
            _ => return Err(format!("unknown family_id: {op_id}")),
        };
        Ok(vec![OutboundMsg {
            text: msg.to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };
        if v.get("event").is_some() {
            return InboundClass::System;
        }

        if let Some(arr) = v.as_array() {
            if arr.len() >= 4 {
                let channel = arr
                    .get(arr.len() - 2)
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
                let pair = arr
                    .get(arr.len() - 1)
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string());
                let fam = if channel.starts_with("ticker") {
                    Some("kraken.public.ws.ticker")
                } else if channel.starts_with("trade") {
                    Some("kraken.public.ws.trade")
                } else if channel.starts_with("book") {
                    Some("kraken.public.ws.book")
                } else if channel.starts_with("ohlc") {
                    Some("kraken.public.ws.ohlc")
                } else if channel.starts_with("spread") {
                    Some("kraken.public.ws.spread")
                } else {
                    None
                };
                if let Some(op) = fam {
                    return InboundClass::Data {
                        op_id: Some(op.into()),
                        symbol: pair,
                        params_canon_hint: Some("{}".into()),
                    };
                }
            }
        }
        InboundClass::System
    }
}
