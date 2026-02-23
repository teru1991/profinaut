use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{fetch_all_symbols, to_exchange_symbol, to_ws_symbol};

#[derive(Debug, Clone)]
pub struct BinanceSpotWsAdapter;
impl BinanceSpotWsAdapter {
    pub fn new() -> Self { Self }
}

fn render_template(mut tpl: String, ws_symbol: &str, params: &Value) -> String {
    tpl = tpl.replace("{symbol}", ws_symbol);
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            if k == "_w" { continue; }
            let ph = format!("{{{k}}}");
            if tpl.contains(&ph) {
                let rep = if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() };
                tpl = tpl.replace(&ph, &rep);
            }
        }
    }
    tpl
}

fn stream_template_for_family(op: &str) -> Option<&'static str> {
    match op {
        "binance.spot.public.ws.trades" => Some("{symbol}@trade"),
        "binance.spot.public.ws.aggTrade" => Some("{symbol}@aggTrade"),
        "binance.spot.public.ws.ticker24h" => Some("{symbol}@ticker"),
        "binance.spot.public.ws.bookTicker" => Some("{symbol}@bookTicker"),
        "binance.spot.public.ws.depth" => Some("{symbol}@depth@{speed}"),
        "binance.spot.public.ws.kline" => Some("{symbol}@kline_{interval}"),
        _ => None,
    }
}

#[async_trait]
impl WsVenueAdapter for BinanceSpotWsAdapter {
    fn exchange_id(&self) -> &str { "binance-spot" }

    fn ws_url(&self) -> String {
        // Spot WS Streams base endpoint (official)
        "wss://stream.binance.com:9443/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn build_subscribe(&self, op_id: &str, symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let tpl = stream_template_for_family(op_id).ok_or_else(|| format!("unknown family_id: {op_id}"))?;
        let raw = to_ws_symbol(&to_exchange_symbol(symbol));
        let stream = render_template(tpl.to_string(), &raw, params);

        // WS Streams SUBSCRIBE
        Ok(vec![OutboundMsg {
            text: json!({"method":"SUBSCRIBE","params":[stream],"id":1}).to_string()
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        // combined stream wrapper or raw stream
        let data = if v.get("stream").is_some() && v.get("data").is_some() {
            v.get("data").cloned().unwrap_or(Value::Null)
        } else {
            v.clone()
        };

        // subscription ack often includes result/id
        if data.get("result").is_some() && data.get("id").is_some() {
            return InboundClass::System;
        }

        let e = data.get("e").and_then(|x| x.as_str()).unwrap_or("");
        let op = match e {
            "trade" => Some("binance.spot.public.ws.trades"),
            "aggTrade" => Some("binance.spot.public.ws.aggTrade"),
            "24hrTicker" => Some("binance.spot.public.ws.ticker24h"),
            "bookTicker" => Some("binance.spot.public.ws.bookTicker"),
            "depthUpdate" => Some("binance.spot.public.ws.depth"),
            "kline" => Some("binance.spot.public.ws.kline"),
            _ => None,
        };

        if let Some(op_id) = op {
            return InboundClass::Data {
                op_id: Some(op_id.to_string()),
                symbol: None,
                params_canon_hint: Some("{}".into()),
            };
        }
        InboundClass::System
    }
}
