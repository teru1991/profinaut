use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::task::block_in_place;
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};

use crate::rest::{GmoCredentials, GmoRest};
use crate::symbols::fetch_symbols;

const WS_PUBLIC_V1: &str = "wss://api.coin.z.com/ws/public/v1";
const WS_PRIVATE_V1_BASE: &str = "wss://api.coin.z.com/ws/private/v1";

fn now_unix_ms() -> u64 {
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()) as u64
}

/// Internal topic encoding for GMO:
/// - Public:  "{channel}|{symbol}"  (+ optional params.option)
/// - Private: "{channel}"          (+ optional params.option)
fn parse_topic(topic: &str) -> Result<(String, Option<String>), String> {
    let parts: Vec<&str> = topic.split('|').collect();
    match parts.as_slice() {
        [ch] => Ok((ch.to_string(), None)),
        [ch, sym] => Ok((ch.to_string(), Some(sym.to_string()))),
        _ => Err(format!("bad _topic format: {topic}")),
    }
}

fn canon_params_hint(topic: &str, option: Option<&str>, weight: u32) -> String {
    // Match planner canon_params ordering: {"_topic":..,"_w":..,"option":..} (option omitted if empty)
    let mut m = std::collections::BTreeMap::<String, Value>::new();
    m.insert("_topic".into(), Value::String(topic.to_string()));
    m.insert("_w".into(), Value::Number((weight as u64).into()));
    if let Some(opt) = option {
        if !opt.is_empty() {
            m.insert("option".into(), Value::String(opt.to_string()));
        }
    }
    let obj: serde_json::Map<String, Value> = m.into_iter().collect();
    Value::Object(obj).to_string()
}

fn weight_for_channel(channel: &str) -> u32 {
    match channel {
        "ticker" => 10,
        "trades" => 20,
        "orderbooks" => 40,
        // private
        "executionEvents" => 10,
        "orderEvents" => 20,
        "positionEvents" => 30,
        "positionSummaryEvents" => 40,
        _ => 90,
    }
}

/// ----------------------------
/// Public adapter
/// ----------------------------
#[derive(Debug, Clone)]
pub struct GmoCoinPublicWsAdapter;

impl GmoCoinPublicWsAdapter {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl WsVenueAdapter for GmoCoinPublicWsAdapter {
    fn exchange_id(&self) -> &str { "gmocoin-public" }

    fn ws_url(&self) -> String { WS_PUBLIC_V1.to_string() }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        fetch_symbols().await
    }

    fn build_subscribe(&self, op_id: &str, symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        // planner_v2 provides params["_topic"] = "channel|symbol"
        let topic = params.get("_topic").and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing _topic (planner_v2 required): op_id={op_id}"))?;

        let (channel, sym_in_topic) = parse_topic(topic)?;
        let sym = sym_in_topic.unwrap_or_else(|| symbol.to_string());

        // trades may include option=TAKER_ONLY  [oai_citation:16‡Coin API](https://api.coin.z.com/docs/)
        let option = params.get("option").and_then(|v| v.as_str()).unwrap_or("");

        let mut msg = json!({
            "command": "subscribe",
            "channel": channel,
            "symbol": sym
        });

        if !option.is_empty() {
            msg.as_object_mut().unwrap().insert("option".into(), Value::String(option.to_string()));
        }

        Ok(vec![OutboundMsg { text: msg.to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        // Most payloads include: channel, symbol
        let channel = v.get("channel").and_then(|x| x.as_str()).unwrap_or("");
        if channel.is_empty() {
            return InboundClass::System;
        }
        let symbol = v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string());

        let topic = match (&symbol, channel) {
            (Some(sym), _) => format!("{channel}|{sym}"),
            (None, _) => channel.to_string(),
        };

        // Option isn't echoed in data, so we only hint "{}" unless we can infer. Keep minimal.
        let w = weight_for_channel(channel);
        let hint = canon_params_hint(&topic, None, w);

        InboundClass::Data {
            op_id: Some(format!("gmocoin.public.ws.{channel}")),
            symbol,
            params_canon_hint: Some(hint),
        }
    }
}

/// ----------------------------
/// Private adapter (token-based)
/// ----------------------------
#[derive(Clone)]
struct TokenState {
    token: Option<String>,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct GmoCoinPrivateWsAdapter {
    rest: GmoRest,
    state: Arc<RwLock<TokenState>>,
}

impl GmoCoinPrivateWsAdapter {
    pub fn new(creds: GmoCredentials) -> Result<Self, String> {
        let rest = GmoRest::new_with_credentials(creds)?;
        Ok(Self {
            rest,
            state: Arc::new(RwLock::new(TokenState {
                token: None,
                expires_at: Instant::now(),
            })),
        })
    }

    fn token_valid(state: &TokenState) -> bool {
        state.token.is_some() && Instant::now() < state.expires_at
    }

    async fn refresh_token_async(&self) -> Result<String, String> {
        let token = self.rest.ws_auth_create().await?;
        // Assume typical 60min; refresh at 55min to be safe (rules also rotates)
        let mut st = self.state.write().unwrap();
        st.token = Some(token.clone());
        st.expires_at = Instant::now() + Duration::from_secs(55 * 60);
        Ok(token)
    }

    fn refresh_token_blocking(&self) -> Result<String, String> {
        // Called from ws_url() which is sync. Use block_in_place to avoid deadlock.
        block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.refresh_token_async())
        })
    }
}

#[async_trait]
impl WsVenueAdapter for GmoCoinPrivateWsAdapter {
    fn exchange_id(&self) -> &str { "gmocoin-private" }

    fn ws_url(&self) -> String {
        // If token is missing/expired, refresh synchronously (trait constraint)
        let needs_refresh = {
            let st = self.state.read().unwrap();
            !Self::token_valid(&st)
        };
        if needs_refresh {
            let _ = self.refresh_token_blocking();
        }

        let token = {
            let st = self.state.read().unwrap();
            st.token.clone().unwrap_or_else(|| "INVALID".to_string())
        };
        format!("{WS_PRIVATE_V1_BASE}/{token}") // docs examples show /v1/{token}  [oai_citation:17‡Coin API](https://api.coin.z.com/docs/)
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        // Private channels in our coverage do not require symbol; return empty.
        // But ensure token is ready at startup.
        let _ = self.refresh_token_async().await?;
        Ok(vec![])
    }

    fn build_subscribe(&self, op_id: &str, _symbol: &str, params: &Value) -> Result<Vec<OutboundMsg>, String> {
        let topic = params.get("_topic").and_then(|v| v.as_str())
            .ok_or_else(|| format!("missing _topic (planner_v2 required): op_id={op_id}"))?;

        let channel = topic;

        let option = params.get("option").and_then(|v| v.as_str()).unwrap_or("");

        let mut msg = json!({
            "command": "subscribe",
            "channel": channel
        });

        if !option.is_empty() {
            msg.as_object_mut().unwrap().insert("option".into(), Value::String(option.to_string()));
        }

        Ok(vec![OutboundMsg { text: msg.to_string() }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: Value = match serde_json::from_slice(raw) {
            Ok(x) => x,
            Err(_) => return InboundClass::Unknown,
        };

        let channel = v.get("channel").and_then(|x| x.as_str()).unwrap_or("");
        if channel.is_empty() {
            return InboundClass::System;
        }

        // positionSummaryEvents may be periodic, but option isn't echoed; keep minimal hint.
        let w = weight_for_channel(channel);
        let hint = canon_params_hint(channel, None, w);

        InboundClass::Data {
            op_id: Some(format!("gmocoin.private.ws.{channel}")),
            symbol: v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string()),
            params_canon_hint: Some(hint),
        }
    }
}