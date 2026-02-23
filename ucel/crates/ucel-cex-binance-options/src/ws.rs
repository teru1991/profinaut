use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};
use crate::symbols::{fetch_all_symbols, to_ws_symbol};

#[derive(Debug, Clone)]
pub struct BinanceOptionsWsAdapter;
impl BinanceOptionsWsAdapter { pub fn new() -> Self { Self } }

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
        "binance.options.public.ws.trades" => Some("{symbol}@trade"),
        "binance.options.public.ws.ticker24h" => Some("{symbol}@ticker"),
        "binance.options.public.ws.depth" => Some("{symbol}@depth@{speed}"),
        "binance.options.public.ws.kline" => Some("{symbol}@kline_{interval}"),
        _ => None,
    }
}

#[async_trait]
impl WsVenueAdapter for BinanceOptionsWsAdapter {
    fn exchange_id(&self) -> &str { "binance-options" }

    fn ws_url(&self) -> String {
        // Options market streams base: wss://fstream.binance.com/public/ (official docs)
        // raw streams: /ws/<streamName>
        "wss://fstream.binance.com/public/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
    }

    fn build_subscribe(&self, op_id: &str, symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let tpl = stream_template_for_family(op_id).ok_or_else(|| format!("unknown family_id: {op_id}"))?;
        let raw = to_ws_symbol(symbol);
        let stream = render_template(tpl.to_string(), &raw, params);
        Ok(vec![OutboundMsg { text: json!({"method":"SUBSCRIBE","params":[stream],"id":1}).to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) { Ok(x) => x, Err(_) => return InboundClass::Unknown };
        let data = if v.get("stream").is_some() && v.get("data").is_some() { v.get("data").cloned().unwrap_or(Value::Null) } else { v.clone() };
        if data.get("result").is_some() && data.get("id").is_some() { return InboundClass::System; }

        let e = data.get("e").and_then(|x| x.as_str()).unwrap_or("");
        let op = match e {
            "trade" => Some("binance.options.public.ws.trades"),
            "24hrTicker" => Some("binance.options.public.ws.ticker24h"),
            "depthUpdate" => Some("binance.options.public.ws.depth"),
            "kline" => Some("binance.options.public.ws.kline"),
            _ => None,
        };

        if let Some(op_id) = op {
            return InboundClass::Data { op_id: Some(op_id.to_string()), symbol: None, params_canon_hint: Some("{}".into()) };
        }
        InboundClass::System
    }
}
