use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OutboundMsg {
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum InboundClass {
    Data {
        op_id: Option<String>,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
    },
    Ack {
        op_id: String,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
    },
    Nack {
        reason: String,
        op_id: Option<String>,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
    },
    System,
    Unknown,
}

#[async_trait]
pub trait WsVenueAdapter: Send + Sync + 'static {
    fn exchange_id(&self) -> &str;
    fn ws_url(&self) -> String;
    async fn fetch_symbols(&self) -> Result<Vec<String>, String>;
    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String>;
    fn classify_inbound(&self, raw: &[u8]) -> InboundClass;

    fn ping_msg(&self) -> Option<OutboundMsg> {
        None
    }
}
