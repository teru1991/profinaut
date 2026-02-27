use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{
    fetch_inverse_symbols, fetch_linear_symbols, fetch_option_symbols, fetch_spot_symbols,
    to_exchange_symbol,
};

#[derive(Debug, Clone)]
enum Kind {
    Spot,
    Linear,
    Inverse,
    Option,
}

#[derive(Debug, Clone)]
pub struct BybitWsAdapter {
    kind: Kind,
}

impl BybitWsAdapter {
    pub fn spot() -> Self {
        Self { kind: Kind::Spot }
    }
    pub fn linear() -> Self {
        Self { kind: Kind::Linear }
    }
    pub fn inverse() -> Self {
        Self {
            kind: Kind::Inverse,
        }
    }
    pub fn option() -> Self {
        Self { kind: Kind::Option }
    }

    fn exchange_id_str(&self) -> &'static str {
        match self.kind {
            Kind::Spot => "bybit-spot",
            Kind::Linear => "bybit-linear",
            Kind::Inverse => "bybit-inverse",
            Kind::Option => "bybit-options",
        }
    }
    fn url(&self) -> &'static str {
        match self.kind {
            Kind::Spot => "wss://stream.bybit.com/v5/public/spot",
            Kind::Linear => "wss://stream.bybit.com/v5/public/linear",
            Kind::Inverse => "wss://stream.bybit.com/v5/public/inverse",
            Kind::Option => "wss://stream.bybit.com/v5/public/option",
        }
    }
}

fn topic_from_params(op_id: &str, symbol: &str, params: &Value) -> Result<String, String> {
    if let Some(t) = params.get("_topic").and_then(|v| v.as_str()) {
        // If planner uses canonical symbol "BTC/USDT" in topic, convert to exchange symbol for Bybit topics.
        // Bybit topics usually use "BTCUSDT" without slash.
        if t.contains("BTC/") || t.contains('/') {
            // replace occurrences after last '.' (common format: "tickers.{symbol}")
            // simplest safe rule:
            // find segments split by '.' and replace any segment containing '/' with exchange symbol.
            let mut segs: Vec<String> = Vec::new();
            for seg in t.split('.') {
                if seg.contains('/') {
                    segs.push(to_exchange_symbol(seg));
                } else {
                    segs.push(seg.to_string());
                }
            }
            return Ok(segs.join("."));
        }
        return Ok(t.to_string());
    }

    // fallback (should not happen with planner_v2)
    if symbol.is_empty() {
        return Err(format!("missing _topic for symbol-less op_id={op_id}"));
    }
    Ok(format!("unknown.{symbol}"))
}

#[async_trait]
impl WsVenueAdapter for BybitWsAdapter {
    fn exchange_id(&self) -> &str {
        self.exchange_id_str()
    }
    fn ws_url(&self) -> String {
        self.url().to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        match self.kind {
            Kind::Spot => fetch_spot_symbols().await,
            Kind::Linear => fetch_linear_symbols().await,
            Kind::Inverse => fetch_inverse_symbols().await,
            Kind::Option => fetch_option_symbols().await,
        }
    }

    fn ping_msg(&self) -> Option<OutboundMsg> {
        Some(OutboundMsg {
            text: json!({"op":"ping"}).to_string(),
        })
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let topic = topic_from_params(op_id, symbol, params)?;
        Ok(vec![OutboundMsg {
            text: json!({"op":"subscribe","args":[topic]}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };
        if v.get("op").and_then(|x| x.as_str()) == Some("pong") {
            return InboundClass::System;
        }
        if v.get("success").is_some() && v.get("op").is_some() {
            return InboundClass::System;
        }

        // Bybit pushes topic field for data
        let topic = v.get("topic").and_then(|x| x.as_str()).unwrap_or("");
        if topic.is_empty() {
            return InboundClass::System;
        }

        InboundClass::Data {
            op_id: None,
            symbol: None,
            params_canon_hint: Some("{}".into()),
        }
    }
}
