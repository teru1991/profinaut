use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};
use crate::symbols::{fetch_all_symbols, to_exchange_symbol, to_ws_symbol};

#[derive(Debug, Clone)]
pub struct BinanceCoinmWsAdapter;
impl BinanceCoinmWsAdapter { pub fn new() -> Self { Self } }

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
        "binance.coinm.public.ws.aggTrade" => Some("{symbol}@aggTrade"),
        "binance.coinm.public.ws.markPrice" => Some("{symbol}@markPrice@1s"),
        "binance.coinm.public.ws.bookTicker" => Some("{symbol}@bookTicker"),
        "binance.coinm.public.ws.depth" => Some("{symbol}@depth@{speed}"),
        "binance.coinm.public.ws.kline" => Some("{symbol}@kline_{interval}"),
        "binance.coinm.public.ws.forceOrder" => Some("{symbol}@forceOrder"),
        "binance.coinm.public.ws.allForceOrder" => Some("!forceOrder@arr"),
        _ => None,
    }
}

#[async_trait]
impl WsVenueAdapter for BinanceCoinmWsAdapter {
    fn exchange_id(&self) -> &str { "binance-coinm" }

    fn ws_url(&self) -> String {
        // COIN-M Futures WS domains: migrate to dstream.binance.com
        "wss://dstream.binance.com/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn build_subscribe(&self, op_id: &str, symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let tpl = stream_template_for_family(op_id).ok_or_else(|| format!("unknown family_id: {op_id}"))?;
        let stream = if tpl.contains("{symbol}") {
            let raw = to_ws_symbol(&to_exchange_symbol(symbol));
            render_template(tpl.to_string(), &raw, params)
        } else {
            tpl.to_string()
        };
        Ok(vec![OutboundMsg { text: json!({"method":"SUBSCRIBE","params":[stream],"id":1}).to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) { Ok(x) => x, Err(_) => return InboundClass::Unknown };
        if let Some(stream) = v.get("stream").and_then(|x| x.as_str()) {
            return InboundClass::Data { op_id: Some(classify_stream_name_coinm(stream)), symbol: None, params_canon_hint: Some("{}".into()) };
        }
        if v.get("result").is_some() && v.get("id").is_some() { return InboundClass::System; }

        let e = v.get("e").and_then(|x| x.as_str()).unwrap_or("");
        let op = match e {
            "aggTrade" => Some("binance.coinm.public.ws.aggTrade"),
            "markPriceUpdate" => Some("binance.coinm.public.ws.markPrice"),
            "bookTicker" => Some("binance.coinm.public.ws.bookTicker"),
            "depthUpdate" => Some("binance.coinm.public.ws.depth"),
            "kline" => Some("binance.coinm.public.ws.kline"),
            "forceOrder" => Some("binance.coinm.public.ws.forceOrder"),
            _ => None,
        };

        if let Some(op_id) = op {
            return InboundClass::Data { op_id: Some(op_id.to_string()), symbol: None, params_canon_hint: Some("{}".into()) };
        }
        InboundClass::System
    }
}

fn classify_stream_name_coinm(stream: &str) -> String {
    if stream == "!forceOrder@arr" { return "binance.coinm.public.ws.allForceOrder".into(); }
    if stream.ends_with("@aggTrade") { return "binance.coinm.public.ws.aggTrade".into(); }
    if stream.contains("@markPrice") { return "binance.coinm.public.ws.markPrice".into(); }
    if stream.ends_with("@bookTicker") { return "binance.coinm.public.ws.bookTicker".into(); }
    if stream.contains("@depth@") { return "binance.coinm.public.ws.depth".into(); }
    if stream.contains("@kline_") { return "binance.coinm.public.ws.kline".into(); }
    if stream.ends_with("@forceOrder") { return "binance.coinm.public.ws.forceOrder".into(); }
    "binance.coinm.public.ws.unknown".into()
}
