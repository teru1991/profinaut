use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{
    fetch_inverse_symbols, fetch_linear_symbols, fetch_option_symbols, fetch_spot_symbols,
    to_exchange_symbol,
};

#[derive(Debug, Clone)]
enum Kind {
    Spot,
    Linear,
    Inverse,
    Option,
}

#[derive(Debug, Clone)]
pub struct BybitWsAdapter {
    kind: Kind,
}

impl BybitWsAdapter {
    pub fn spot() -> Self {
        Self { kind: Kind::Spot }
    }
    pub fn linear() -> Self {
        Self { kind: Kind::Linear }
    }
    pub fn inverse() -> Self {
        Self {
            kind: Kind::Inverse,
        }
    }
    pub fn option() -> Self {
        Self { kind: Kind::Option }
    }

    fn exchange_id_str(&self) -> &'static str {
        match self.kind {
            Kind::Spot => "bybit-spot",
            Kind::Linear => "bybit-linear",
            Kind::Inverse => "bybit-inverse",
            Kind::Option => "bybit-options",
        }
    }
    fn url(&self) -> &'static str {
        match self.kind {
            Kind::Spot => "wss://stream.bybit.com/v5/public/spot",
            Kind::Linear => "wss://stream.bybit.com/v5/public/linear",
            Kind::Inverse => "wss://stream.bybit.com/v5/public/inverse",
            Kind::Option => "wss://stream.bybit.com/v5/public/option",
        }
    }
}

fn render_template(mut s: String, symbol: &str, params: &Value) -> String {
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

fn topic_template_for_family(family: &str) -> Option<&'static str> {
    match family {
        "bybit.public.ws.tickers" => Some("tickers.{symbol}"),
        "bybit.public.ws.publicTrade" => Some("publicTrade.{symbol}"),
        "bybit.public.ws.orderbook" => Some("orderbook.{depth}.{symbol}"),
        "bybit.public.ws.kline" => Some("kline.{interval}.{symbol}"),
        "bybit.public.ws.allLiquidation" => Some("allLiquidation.{symbol}"),
        "bybit.public.ws.openInterest" => Some("openInterest.{symbol}"),
        "bybit.public.ws.funding" => Some("funding.{symbol}"),
        "bybit.public.ws.adl-alert" => Some("adl.alert"),
        "bybit.public.ws.insurance" => Some("insurance"),
        _ => None,
    }
}

fn params_hint_from_topic(topic: &str) -> String {
    let parts: Vec<&str> = topic.split('.').collect();
    if topic.starts_with("orderbook.") && parts.len() >= 3 {
        return json!({"depth": parts[1]}).to_string();
    }
    if topic.starts_with("kline.") && parts.len() >= 3 {
        return json!({"interval": parts[1]}).to_string();
    }
    "{}".into()
}

#[async_trait]
impl WsVenueAdapter for BybitWsAdapter {
    fn exchange_id(&self) -> &str {
        self.exchange_id_str()
    }
    fn ws_url(&self) -> String {
        self.url().to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        match self.kind {
            Kind::Spot => fetch_spot_symbols().await,
            Kind::Linear => fetch_linear_symbols().await,
            Kind::Inverse => fetch_inverse_symbols().await,
            Kind::Option => fetch_option_symbols().await,
        }
    }

    fn ping_msg(&self) -> Option<OutboundMsg> {
        Some(OutboundMsg {
            text: json!({"op":"ping"}).to_string(),
        })
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        let tpl = topic_template_for_family(op_id)
            .ok_or_else(|| format!("unknown family_id: {op_id}"))?;
        let sym = to_exchange_symbol(symbol);
        let topic = if tpl.contains("{symbol}") {
            render_template(tpl.to_string(), &sym, params)
        } else {
            tpl.to_string()
        };
        Ok(vec![OutboundMsg {
            text: json!({"op":"subscribe","args":[topic]}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };
        if v.get("op").and_then(|x| x.as_str()) == Some("pong") {
            return InboundClass::System;
        }
        if v.get("success").is_some() && v.get("op").is_some() {
            return InboundClass::System;
        }

        let topic = v.get("topic").and_then(|x| x.as_str()).unwrap_or("");
        if v.get("data").is_none() {
            return InboundClass::System;
        }

        let fam = if topic.starts_with("tickers.") {
            Some("bybit.public.ws.tickers")
        } else if topic.starts_with("publicTrade.") {
            Some("bybit.public.ws.publicTrade")
        } else if topic.starts_with("orderbook.") {
            Some("bybit.public.ws.orderbook")
        } else if topic.starts_with("kline.") {
            Some("bybit.public.ws.kline")
        } else if topic.starts_with("allLiquidation.") {
            Some("bybit.public.ws.allLiquidation")
        } else if topic.starts_with("openInterest.") {
            Some("bybit.public.ws.openInterest")
        } else if topic.starts_with("funding.") {
            Some("bybit.public.ws.funding")
        } else if topic.contains("adl") {
            Some("bybit.public.ws.adl-alert")
        } else if topic.contains("insurance") {
            Some("bybit.public.ws.insurance")
        } else {
            None
        };

        if let Some(op) = fam {
            return InboundClass::Data {
                op_id: Some(op.into()),
                symbol: None,
                params_canon_hint: Some(params_hint_from_topic(topic)),
            };
        }
        InboundClass::System
    }
}
