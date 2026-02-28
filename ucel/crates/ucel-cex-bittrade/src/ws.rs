use async_trait::async_trait;
use flate2::read::GzDecoder;
use serde_json::{json, Value};
use std::io::Read;
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::fetch_all_symbols;

#[derive(Debug, Clone)]
pub struct BitTradeWsAdapter;

impl Default for BitTradeWsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BitTradeWsAdapter {
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
        "bittrade.public.ws.detail" => Some("market.{symbol}.detail"),
        "bittrade.public.ws.trades" => Some("market.{symbol}.trade.detail"),
        "bittrade.public.ws.depth" => Some("market.{symbol}.depth.{type}"),
        "bittrade.public.ws.kline" => Some("market.{symbol}.kline.{period}"),
        "bittrade.public.ws.bbo" => Some("market.{symbol}.bbo"),
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
impl WsVenueAdapter for BitTradeWsAdapter {
    fn exchange_id(&self) -> &str {
        "bittrade"
    }
    fn ws_url(&self) -> String {
        "wss://ws-api.bittrade.co.jp/ws".to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_all_symbols().await
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

        let fam = if ch.ends_with(".detail") && !ch.contains(".trade.") {
            Some("bittrade.public.ws.detail")
        } else if ch.ends_with(".trade.detail") {
            Some("bittrade.public.ws.trades")
        } else if ch.contains(".depth.") {
            Some("bittrade.public.ws.depth")
        } else if ch.contains(".kline.") {
            Some("bittrade.public.ws.kline")
        } else if ch.ends_with(".bbo") {
            Some("bittrade.public.ws.bbo")
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
