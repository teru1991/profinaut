use async_trait::async_trait;
use flate2::read::GzDecoder;
use serde_json::{json, Value};
use std::io::Read;
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::fetch_spot_symbols;

#[derive(Debug, Clone, Default)]
pub struct HtxSpotWsAdapter;
impl HtxSpotWsAdapter {
    pub fn new() -> Self {
        Self
    }
}

fn gunzip_if_needed(raw: &[u8]) -> Vec<u8> {
    if raw.len() >= 3 && raw[0] == 0x1f && raw[1] == 0x8b && raw[2] == 0x08 {
        let mut d = GzDecoder::new(raw);
        let mut out = Vec::new();
        if d.read_to_end(&mut out).is_ok() {
            return out;
        }
    }
    raw.to_vec()
}

fn template_for_family(op: &str) -> Option<&'static str> {
    match op {
        "htx.public.ws.ticker" => Some("market.{symbol}.ticker"),
        "htx.public.ws.trades" => Some("market.{symbol}.trade.detail"),
        "htx.public.ws.depth" => Some("market.{symbol}.depth.{type}"),
        "htx.public.ws.mbp" => Some("market.{symbol}.mbp.{levels}"),
        "htx.public.ws.bbo" => Some("market.{symbol}.bbo"),
        "htx.public.ws.kline" => Some("market.{symbol}.kline.{period}"),
        _ => None,
    }
}

fn render(mut s: String, symbol: &str, params: &Value) -> String {
    s = s.replace("{symbol}", symbol);
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            if k == "_w" {
                continue;
            }
            let ph = format!("{{{k}}}");
            if s.contains(&ph) {
                let rep = if v.is_string() {
                    v.as_str().unwrap_or_default().to_string()
                } else {
                    v.to_string()
                };
                s = s.replace(&ph, &rep);
            }
        }
    }
    s
}

#[async_trait]
impl WsVenueAdapter for HtxSpotWsAdapter {
    fn exchange_id(&self) -> &str {
        "htx-spot"
    }
    fn ws_url(&self) -> String {
        "wss://api.huobi.pro/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_spot_symbols().await
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let tpl =
            template_for_family(op_id).ok_or_else(|| format!("unknown family_id: {op_id}"))?;
        let sub = render(tpl.to_string(), symbol, params);
        Ok(vec![OutboundMsg {
            text: json!({"sub": sub, "id":"1"}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let bytes = gunzip_if_needed(raw);
        let v: Value = match serde_json::from_slice(&bytes) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        if let Some(ts) = v.get("ping").and_then(|x| x.as_i64()) {
            return InboundClass::Respond {
                msg: OutboundMsg {
                    text: json!({"pong": ts}).to_string(),
                },
            };
        }

        let ch = v.get("ch").and_then(|x| x.as_str()).unwrap_or("");
        if ch.is_empty() {
            return InboundClass::System;
        }

        let fam = if ch.ends_with(".ticker") {
            Some("htx.public.ws.ticker")
        } else if ch.ends_with(".trade.detail") {
            Some("htx.public.ws.trades")
        } else if ch.contains(".depth.") {
            Some("htx.public.ws.depth")
        } else if ch.contains(".mbp.") {
            Some("htx.public.ws.mbp")
        } else if ch.ends_with(".bbo") {
            Some("htx.public.ws.bbo")
        } else if ch.contains(".kline.") {
            Some("htx.public.ws.kline")
        } else {
            None
        };

        if let Some(op) = fam {
            return InboundClass::Data {
                op_id: Some(op.into()),
                symbol: None,
                params_canon_hint: Some("{}".into()),
            };
        }
        InboundClass::System
    }
}
