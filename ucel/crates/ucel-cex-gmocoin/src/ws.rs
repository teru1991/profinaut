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
        // official crypto public ws endpoint: wss://api.coin.z.com/ws/public/v1
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
        // coverage SSOT (ucel/coverage/gmocoin.yaml) に完全一致させる
        let channel = match op_id {
            "crypto.public.ws.ticker.update" => "ticker",
            "crypto.public.ws.trades.update" => "trades",
            "crypto.public.ws.orderbooks.update" => "orderbooks",
            // v1.0 ingest は public のみ。private / fx は別workerに分離するのが安全
            _ => return Err(format!("unsupported op_id={op_id}")),
        };

        // GMOは "BTC_JPY" 等の raw symbol を要求する
        let ex_symbol = to_exchange_symbol(symbol);

        // official request format: {"command":"subscribe","channel":"...","symbol":"..."}
        let msg = json!({
            "command": "subscribe",
            "channel": channel,
            "symbol": ex_symbol
        });

        Ok(vec![OutboundMsg {
            text: msg.to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        // heavy parse禁止：必要最小
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        // 代表例（公式例・周辺実装）では channel / symbol を持つ
        let channel = v.get("channel").and_then(|x| x.as_str()).unwrap_or("");
        let symbol_raw = v
            .get("symbol")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        // raw -> canonical（BTC_JPY -> BTC/JPY）
        let symbol = symbol_raw.map(|s| {
            if s.contains('_') {
                s.replace('_', "/")
            } else {
                s
            }
        });

        // channel -> coverage op_id（SSOTに完全一致）
        let op_id = match channel {
            "ticker" => Some("crypto.public.ws.ticker.update".to_string()),
            "trades" => Some("crypto.public.ws.trades.update".to_string()),
            "orderbooks" => Some("crypto.public.ws.orderbooks.update".to_string()),
            _ => None,
        };

        // Public WSは ACK が無い/弱いことがあるため Data で active 確定できる形にする
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

    fn ping_msg(&self) -> Option<OutboundMsg> {
        // GMO crypto public ws は明示ping不要のことが多いので None
        None
    }
}
