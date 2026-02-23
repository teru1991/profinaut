use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{fetch_symbols_by_inst_type, to_exchange_symbol};

#[derive(Debug, Clone)]
pub struct OkxWsAdapter {
    exchange_id: &'static str,
    inst_type: &'static str,
}
impl OkxWsAdapter {
    pub fn spot() -> Self { Self { exchange_id: "okx-spot", inst_type: "SPOT" } }
    pub fn swap() -> Self { Self { exchange_id: "okx-swap", inst_type: "SWAP" } }
    pub fn futures() -> Self { Self { exchange_id: "okx-futures", inst_type: "FUTURES" } }
    pub fn option() -> Self { Self { exchange_id: "okx-option", inst_type: "OPTION" } }
}

#[async_trait]
impl WsVenueAdapter for OkxWsAdapter {
    fn exchange_id(&self) -> &str { self.exchange_id }
    fn ws_url(&self) -> String { "wss://ws.okx.com:8443/ws/v5/public".to_string() }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> { fetch_symbols_by_inst_type(self.inst_type).await }
    fn ping_msg(&self) -> Option<OutboundMsg> { Some(OutboundMsg { text: "ping".into() }) }

    fn build_subscribe(&self, op_id: &str, symbol: &str, _params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let channel = op_id.strip_prefix("okx.public.ws.").ok_or("bad family_id")?;
        if symbol.is_empty() {
            let msg = json!({"op":"subscribe","args":[{"channel":channel}]});
            return Ok(vec![OutboundMsg { text: msg.to_string() }]);
        }
        let inst = to_exchange_symbol(symbol);
        let msg = json!({"op":"subscribe","args":[{"channel":channel,"instId":inst}]});
        Ok(vec![OutboundMsg { text: msg.to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        if raw == b"pong" { return InboundClass::System; }
        let v: Value = match serde_json::from_slice(raw) { Ok(x) => x, Err(_) => return InboundClass::Unknown };
        if v.get("event").is_some() { return InboundClass::System; }
        let chan = v.get("arg").and_then(|a| a.get("channel")).and_then(|x| x.as_str()).unwrap_or("");
        if chan.is_empty() { return InboundClass::System; }
        InboundClass::Data { op_id: Some(format!("okx.public.ws.{chan}")), symbol: None, params_canon_hint: Some("{}".into()) }
    }
}
