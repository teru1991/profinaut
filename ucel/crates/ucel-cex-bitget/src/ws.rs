use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{fetch_all_symbols, to_exchange_symbol};

#[derive(Debug, Clone)]
pub struct BitgetSpotWsAdapter;
impl BitgetSpotWsAdapter { pub fn new() -> Self { Self } }

#[async_trait]
impl WsVenueAdapter for BitgetSpotWsAdapter {
    fn exchange_id(&self) -> &str { "bitget-spot" }
    fn ws_url(&self) -> String { "wss://ws.bitget.com/v2/ws/public".to_string() }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> { fetch_all_symbols().await }

    fn ping_msg(&self) -> Option<OutboundMsg> { Some(OutboundMsg { text: "ping".into() }) }

    fn build_subscribe(&self, op_id: &str, symbol: &str, _params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let channel = op_id.strip_prefix("bitget.public.ws.").ok_or("bad family_id")?;
        let inst_id = if symbol.contains('/') { to_exchange_symbol(symbol) } else { symbol.to_string() };
        let msg = json!({"op":"subscribe","args":[{"instType":"SPOT","channel":channel,"instId":inst_id}]});
        Ok(vec![OutboundMsg { text: msg.to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        if raw == b"pong" || raw == b"pong\n" { return InboundClass::System; }
        let v: Value = match serde_json::from_slice(raw) { Ok(x) => x, Err(_) => return InboundClass::Unknown };
        if v.get("event").is_some() { return InboundClass::System; }
        let chan = v.get("arg").and_then(|a| a.get("channel")).and_then(|x| x.as_str()).unwrap_or("");
        if chan.is_empty() { return InboundClass::System; }
        InboundClass::Data { op_id: Some(format!("bitget.public.ws.{chan}")), symbol: None, params_canon_hint: Some("{}".into()) }
    }
}
