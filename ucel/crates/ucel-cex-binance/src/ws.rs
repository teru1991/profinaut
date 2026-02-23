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
        "binance.spot.public.ws.trade" => Some("{symbol}@trade"),
        "binance.spot.public.ws.aggTrade" => Some("{symbol}@aggTrade"),
        "binance.spot.public.ws.ticker24h" => Some("{symbol}@ticker"),
        "binance.spot.public.ws.miniTicker" => Some("{symbol}@miniTicker"),
        "binance.spot.public.ws.bookTicker" => Some("{symbol}@bookTicker"),
        "binance.spot.public.ws.depth" => Some("{symbol}@depth@{speed}"),
        "binance.spot.public.ws.kline" => Some("{symbol}@kline_{interval}"),

        "binance.spot.public.ws.allBookTicker" => Some("!bookTicker"),
        "binance.spot.public.ws.allTicker24h" => Some("!ticker@arr"),
        "binance.spot.public.ws.allMiniTicker" => Some("!miniTicker@arr"),
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

        // symbol-less: template has no {symbol}
        let stream = if tpl.contains("{symbol}") {
            let raw = to_ws_symbol(&to_exchange_symbol(symbol));
            render_template(tpl.to_string(), &raw, params)
        } else {
            tpl.to_string()
        };

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

        // combined stream wrapper: {"stream":"xxx","data":{...}}
        if let Some(stream) = v.get("stream").and_then(|x| x.as_str()) {
            return InboundClass::Data {
                op_id: Some(classify_stream_name_spot(stream)),
                symbol: None,
                params_canon_hint: Some("{}".into()),
            };
        }

        // subscription ack often includes result/id
        if v.get("result").is_some() && v.get("id").is_some() {
            return InboundClass::System;
        }

        // raw payload: best-effort by "e"
        let e = v.get("e").and_then(|x| x.as_str()).unwrap_or("");
        let op = match e {
            "trade" => "binance.spot.public.ws.trade",
            "aggTrade" => "binance.spot.public.ws.aggTrade",
            "24hrTicker" => "binance.spot.public.ws.ticker24h",
            "24hrMiniTicker" => "binance.spot.public.ws.miniTicker",
            "bookTicker" => "binance.spot.public.ws.bookTicker",
            "depthUpdate" => "binance.spot.public.ws.depth",
            "kline" => "binance.spot.public.ws.kline",
            _ => "",
        };
        if op.is_empty() {
            InboundClass::System
        } else {
            InboundClass::Data { op_id: Some(op.into()), symbol: None, params_canon_hint: Some("{}".into()) }
        }
    }
}

fn classify_stream_name_spot(stream: &str) -> String {
    if stream == "!bookTicker" { return "binance.spot.public.ws.allBookTicker".into(); }
    if stream == "!ticker@arr" { return "binance.spot.public.ws.allTicker24h".into(); }
    if stream == "!miniTicker@arr" { return "binance.spot.public.ws.allMiniTicker".into(); }

    if stream.ends_with("@trade") { return "binance.spot.public.ws.trade".into(); }
    if stream.ends_with("@aggTrade") { return "binance.spot.public.ws.aggTrade".into(); }
    if stream.ends_with("@ticker") { return "binance.spot.public.ws.ticker24h".into(); }
    if stream.ends_with("@miniTicker") { return "binance.spot.public.ws.miniTicker".into(); }
    if stream.ends_with("@bookTicker") { return "binance.spot.public.ws.bookTicker".into(); }
    if stream.contains("@depth@") { return "binance.spot.public.ws.depth".into(); }
    if stream.contains("@kline_") { return "binance.spot.public.ws.kline".into(); }

    "binance.spot.public.ws.unknown".into()
}
