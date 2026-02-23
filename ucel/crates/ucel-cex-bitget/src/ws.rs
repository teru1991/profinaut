use async_trait::async_trait;
use serde_json::{json, Value};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::symbols::{
    fetch_coin_futures_symbols, fetch_spot_symbols, fetch_usdc_futures_symbols,
    fetch_usdt_futures_symbols,
};

/// Bitget WS domains (public)
/// wss://ws.bitget.com/v2/ws/public :contentReference[oaicite:7]{index=7}
const WS_PUBLIC_URL: &str = "wss://ws.bitget.com/v2/ws/public";

#[derive(Debug, Clone, Copy)]
pub enum BitgetKind {
    Spot,
    UsdtFutures,
    CoinFutures,
    UsdcFutures,
}

#[derive(Debug, Clone)]
pub struct BitgetWsAdapter {
    kind: BitgetKind,
}

impl BitgetWsAdapter {
    pub fn spot() -> Self {
        Self { kind: BitgetKind::Spot }
    }
    pub fn usdt_futures() -> Self {
        Self { kind: BitgetKind::UsdtFutures }
    }
    pub fn coin_futures() -> Self {
        Self { kind: BitgetKind::CoinFutures }
    }
    pub fn usdc_futures() -> Self {
        Self { kind: BitgetKind::UsdcFutures }
    }

    fn exchange_id_str(&self) -> &'static str {
        match self.kind {
            BitgetKind::Spot => "bitget-spot",
            BitgetKind::UsdtFutures => "bitget-usdt-futures",
            BitgetKind::CoinFutures => "bitget-coin-futures",
            BitgetKind::UsdcFutures => "bitget-usdc-futures",
        }
    }

    fn inst_type(&self) -> &'static str {
        match self.kind {
            BitgetKind::Spot => "SPOT",
            BitgetKind::UsdtFutures => "USDT-FUTURES",
            BitgetKind::CoinFutures => "COIN-FUTURES",
            BitgetKind::UsdcFutures => "USDC-FUTURES",
        }
    }

    /// Our internal SSOT topic encoding: "INSTTYPE|channel|instId"
    /// - planner_v2 puts final string into params["_topic"]
    fn parse_topic(topic: &str) -> Result<(&str, &str, &str), String> {
        let parts: Vec<&str> = topic.split('|').collect();
        if parts.len() != 3 {
            return Err(format!("bad _topic format (expected 3 parts): {topic}"));
        }
        Ok((parts[0], parts[1], parts[2]))
    }

    /// For active marking, we must reconstruct params_canon_hint that matches planner's canon_params.
    /// planner includes:
    /// - "_topic": "INSTTYPE|channel|instId"
    /// - "_w": weight (we set weights in YAML)
    /// - plus "interval" for candles (because YAML has params.interval)
    fn params_canon_hint_for(inst_type: &str, channel: &str, inst_id: &str) -> String {
        let (op_id, interval_opt, weight) = opid_interval_weight(inst_type, channel);

        // build a minimal params object that matches planner seed keys
        let mut obj = serde_json::Map::new();
        obj.insert("_topic".into(), Value::String(format!("{inst_type}|{channel}|{inst_id}")));
        obj.insert("_w".into(), Value::Number((weight as u64).into()));
        if let Some(interval) = interval_opt {
            obj.insert("interval".into(), Value::String(interval.to_string()));
        }

        // IMPORTANT: keys must be sorted like canon_params does (BTreeMap) and serialized as JSON object string.
        // We'll mimic canon_params ordering by building BTreeMap.
        let mut sorted = std::collections::BTreeMap::<String, Value>::new();
        for (k, v) in obj {
            sorted.insert(k, v);
        }
        let obj_sorted: serde_json::Map<String, Value> = sorted.into_iter().collect();
        Value::Object(obj_sorted).to_string()
    }

    fn build_subscribe_payload(&self, topic: &str) -> Result<String, String> {
        let (inst_type, channel, inst_id) = Self::parse_topic(topic)?;
        // Subscribe args list<Object>: {instType, channel, instId} :contentReference[oaicite:8]{index=8}
        Ok(json!({
            "op": "subscribe",
            "args": [{
                "instType": inst_type,
                "channel": channel,
                "instId": inst_id
            }]
        })
            .to_string())
    }
}

/// Decide op_id / interval / weight from (instType, channel).
/// Must match coverage_v2 IDs and weights we published.
fn opid_interval_weight(inst_type: &str, channel: &str) -> (String, Option<String>, u32) {
    let inst = inst_type.to_lowercase().replace('-', "_");

    // candles: channel looks like "candle1m", "candle6Hutc", etc
    if let Some(rest) = channel.strip_prefix("candle") {
        // rest is interval string exactly as YAML param values
        let op = format!("bitget.{inst}.public.ws.candles");
        return (op, Some(rest.to_string()), 60);
    }

    // books variants
    let (op, w) = match channel {
        "ticker" => (format!("bitget.{inst}.public.ws.ticker"), 10),
        "trade" => (format!("bitget.{inst}.public.ws.trade"), 20),

        "books" => (format!("bitget.{inst}.public.ws.books"), 40),
        "books1" => (format!("bitget.{inst}.public.ws.books1"), 41),
        "books5" => (format!("bitget.{inst}.public.ws.books5"), 42),
        "books15" => (format!("bitget.{inst}.public.ws.books15"), 43),

        _ => (format!("bitget.{inst}.public.ws.unknown"), 90),
    };

    (op, None, w)
}

#[async_trait]
impl WsVenueAdapter for BitgetWsAdapter {
    fn exchange_id(&self) -> &str {
        self.exchange_id_str()
    }

    fn ws_url(&self) -> String {
        WS_PUBLIC_URL.to_string()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        match self.kind {
            BitgetKind::Spot => fetch_spot_symbols().await,
            BitgetKind::UsdtFutures => fetch_usdt_futures_symbols().await,
            BitgetKind::CoinFutures => fetch_coin_futures_symbols().await,
            BitgetKind::UsdcFutures => fetch_usdc_futures_symbols().await,
        }
    }

    /// Bitget requires:
    /// - send string "ping" every 30 seconds
    /// - expect string "pong"
    /// - server disconnects if no "ping" within 2 minutes
    /// - websocket forcibly disconnected every 24 hours
    /// - accept up to 10 messages/sec :contentReference[oaicite:9]{index=9}
    fn ping_msg(&self) -> Option<OutboundMsg> {
        Some(OutboundMsg { text: "ping".to_string() })
    }

    fn build_subscribe(&self, op_id: &str, _symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let topic = params
            .get("_topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing params._topic (planner_v2 required): op_id={op_id}"))?;

        // Ensure topic instType matches adapter kind (safety)
        let (inst_type, _channel, _inst_id) = Self::parse_topic(topic)?;
        if inst_type != self.inst_type() {
            return Err(format!(
                "instType mismatch: adapter={} expects {} but got {} in _topic={}",
                self.exchange_id_str(),
                self.inst_type(),
                inst_type,
                topic
            ));
        }

        let payload = self.build_subscribe_payload(topic)?;
        Ok(vec![OutboundMsg { text: payload }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        // Bitget "pong" is string
        if raw == b"pong" {
            return InboundClass::System;
        }

        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => {
                // sometimes it is text like "pong"
                if let Ok(s) = std::str::from_utf8(raw) {
                    if s.trim() == "pong" {
                        return InboundClass::System;
                    }
                }
                return InboundClass::Unknown;
            }
        };

        // subscribe response example uses "event":"subscribe" / "event":"error" (docs show this pattern).
        // We treat it as Ack/Nack to activate subscriptions quickly.
        if let Some(event) = v.get("event").and_then(|x| x.as_str()) {
            if event == "error" {
                let msg = v.get("msg").and_then(|x| x.as_str()).unwrap_or("error").to_string();
                return InboundClass::Nack { reason: msg, op_id: None, symbol: None, params_canon_hint: None };
            }
            if event == "subscribe" || event == "unsubscribe" {
                // best-effort parse arg to mark active
                if let Some(arg) = v.get("arg").and_then(|x| x.as_object()) {
                    let inst_type = arg.get("instType").and_then(|x| x.as_str()).unwrap_or("");
                    let channel = arg.get("channel").and_then(|x| x.as_str()).unwrap_or("");
                    let inst_id = arg.get("instId").and_then(|x| x.as_str()).unwrap_or("");

                    if !inst_type.is_empty() && !channel.is_empty() {
                        let (op, _interval, _w) = opid_interval_weight(inst_type, channel);
                        let hint = params_canon_hint_for(inst_type, channel, inst_id);
                        return InboundClass::Ack {
                            op_id: op,
                            symbol: if inst_id.is_empty() { None } else { Some(inst_id.to_string()) },
                            params_canon_hint: Some(hint),
                        };
                    }
                }
                return InboundClass::System;
            }
        }

        // data format includes "arg":{instType,channel,instId} and "data": [...]
        if let Some(arg) = v.get("arg").and_then(|x| x.as_object()) {
            let inst_type = arg.get("instType").and_then(|x| x.as_str()).unwrap_or("");
            let channel = arg.get("channel").and_then(|x| x.as_str()).unwrap_or("");
            let inst_id = arg.get("instId").and_then(|x| x.as_str()).unwrap_or("");

            if !inst_type.is_empty() && !channel.is_empty() {
                let (op, _interval, _w) = opid_interval_weight(inst_type, channel);
                let hint = params_canon_hint_for(inst_type, channel, inst_id);
                return InboundClass::Data {
                    op_id: Some(op),
                    symbol: if inst_id.is_empty() { None } else { Some(inst_id.to_string()) },
                    params_canon_hint: Some(hint),
                };
            }
        }

        InboundClass::System
    }
}