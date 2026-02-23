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

fn render(mut tpl: String, symbol: &str, params: &Value) -> String {
    tpl = tpl.replace("{symbol}", symbol);
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            if k == "_w" {
                continue;
            }
            let ph = format!("{{{k}}}");
            if tpl.contains(&ph) {
                let rep = if v.is_string() {
                    v.as_str().unwrap().to_string()
                } else {
                    v.to_string()
                };
                tpl = tpl.replace(&ph, &rep);
            }
        }
    }
    tpl
}

fn topic_template_for_family(family: &str) -> Option<&'static str> {
    match family {
        // public core
        "bybit.public.ws.tickers" => Some("tickers.{symbol}"),
        "bybit.public.ws.publicTrade" => Some("publicTrade.{symbol}"),
        "bybit.public.ws.orderbook" => Some("orderbook.{depth}.{symbol}"),
        "bybit.public.ws.orderbook_rpi" => Some("orderbook.rpi.{symbol}"),
        "bybit.public.ws.kline" => Some("kline.{interval}.{symbol}"),
        "bybit.public.ws.allLiquidation" => Some("allLiquidation.{symbol}"),
        "bybit.public.ws.priceLimit" => Some("priceLimit.{symbol}"),
        // symbol-less
        "bybit.public.ws.insurance" => Some("insurance.{group}"),
        "bybit.public.ws.insurance_inverse" => Some("insurance.inverse"),
        "bybit.public.ws.adlAlert" => Some("adlAlert.{coin}"),
        _ => None,
    }
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

        // symbol-less topics: ignore `symbol` argument
        let topic = if tpl.contains("{symbol}") {
            let sym = to_exchange_symbol(symbol);
            render(tpl.to_string(), &sym, params)
        } else {
            render(tpl.to_string(), "", params)
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
        if topic.is_empty() {
            return InboundClass::System;
        }

        let fam = if topic.starts_with("tickers.") {
            Some("bybit.public.ws.tickers")
        } else if topic.starts_with("publicTrade.") {
            Some("bybit.public.ws.publicTrade")
        } else if topic.starts_with("orderbook.rpi.") {
            Some("bybit.public.ws.orderbook_rpi")
        } else if topic.starts_with("orderbook.") {
            Some("bybit.public.ws.orderbook")
        } else if topic.starts_with("kline.") {
            Some("bybit.public.ws.kline")
        } else if topic.starts_with("allLiquidation.") {
            Some("bybit.public.ws.allLiquidation")
        } else if topic.starts_with("insurance.USDT") || topic.starts_with("insurance.USDC") {
            Some("bybit.public.ws.insurance")
        } else if topic == "insurance.inverse" {
            Some("bybit.public.ws.insurance_inverse")
        } else if topic.starts_with("priceLimit.") {
            Some("bybit.public.ws.priceLimit")
        } else if topic.starts_with("adlAlert.") {
            Some("bybit.public.ws.adlAlert")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_subscribe_supports_symbol_less_family() {
        let a = BybitWsAdapter::linear();
        let msgs = a
            .build_subscribe("bybit.public.ws.insurance", "", &json!({"group":"USDT"}))
            .unwrap();
        let v: Value = serde_json::from_str(&msgs[0].text).unwrap();
        assert_eq!(v["args"][0], "insurance.USDT");
    }

    #[test]
    fn classify_inbound_supports_new_topics() {
        let a = BybitWsAdapter::linear();
        let insurance = json!({"topic":"insurance.USDC","data":{}}).to_string();
        let adl = json!({"topic":"adlAlert.USDT","data":{}}).to_string();

        match a.classify_inbound(insurance.as_bytes()) {
            InboundClass::Data { op_id, .. } => {
                assert_eq!(op_id.as_deref(), Some("bybit.public.ws.insurance"))
            }
            _ => panic!("expected data classification for insurance topic"),
        }
        match a.classify_inbound(adl.as_bytes()) {
            InboundClass::Data { op_id, .. } => {
                assert_eq!(op_id.as_deref(), Some("bybit.public.ws.adlAlert"))
            }
            _ => panic!("expected data classification for adlAlert topic"),
        }
    }
}
