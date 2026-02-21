pub mod config;
pub mod coverage;
pub mod errors;
pub mod registry;

use bytes::Bytes;
use futures_util::{SinkExt, Stream, StreamExt};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use ucel_transport::{next_retry_delay_ms, RetryPolicy};

pub use config::InvokerConfig;
pub use errors::InvokerError;
pub use registry::{OperationKind, ResolvedSpec, SpecRegistry};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VenueId(String);

impl VenueId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Display for VenueId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for VenueId {
    type Err = InvokerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_ascii_lowercase();
        if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(InvokerError::InvalidVenueId(s));
        }
        Ok(Self(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(String);
impl OperationId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Display for OperationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for OperationId {
    type Err = InvokerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty()
            || !s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(InvokerError::InvalidOperationId(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketKind {
    Spot,
    Perp,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSymbol {
    pub kind: MarketKind,
    pub base: String,
    pub quote: String,
}
impl Display for MarketSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            MarketKind::Spot => "SPOT",
            MarketKind::Perp => "PERP",
        };
        write!(f, "{kind}:{}/{}", self.base, self.quote)
    }
}
impl FromStr for MarketSymbol {
    type Err = InvokerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, pair) = s
            .split_once(':')
            .ok_or_else(|| InvokerError::InvalidMarketSymbol(s.to_string()))?;
        let (base, quote) = pair
            .split_once('/')
            .ok_or_else(|| InvokerError::InvalidMarketSymbol(s.to_string()))?;
        let norm = |x: &str| {
            let t = x.trim().to_ascii_uppercase();
            if t.is_empty() || !t.chars().all(|c| c.is_ascii_alphanumeric()) {
                None
            } else {
                Some(t)
            }
        };
        let base = norm(base).ok_or_else(|| InvokerError::InvalidMarketSymbol(s.to_string()))?;
        let quote = norm(quote).ok_or_else(|| InvokerError::InvalidMarketSymbol(s.to_string()))?;
        let kind = match kind.trim().to_ascii_uppercase().as_str() {
            "SPOT" => MarketKind::Spot,
            "PERP" => MarketKind::Perp,
            _ => return Err(InvokerError::InvalidMarketSymbol(s.to_string())),
        };
        Ok(Self { kind, base, quote })
    }
}

pub struct SymbolCodec;
impl SymbolCodec {
    pub fn encode(venue: &VenueId, symbol: &MarketSymbol) -> String {
        let v = venue.as_str();
        if v == "coincheck" || v == "bitbank" {
            return format!(
                "{}_{}",
                symbol.base.to_ascii_lowercase(),
                symbol.quote.to_ascii_lowercase()
            );
        }
        match symbol.kind {
            MarketKind::Spot => format!("{}{}", symbol.base, symbol.quote),
            MarketKind::Perp => format!("{}{}", symbol.base, symbol.quote),
        }
    }

    pub fn decode(_venue: &VenueId, raw: &str) -> Option<MarketSymbol> {
        if let Some((b, q)) = raw.split_once('_') {
            return Some(MarketSymbol {
                kind: MarketKind::Spot,
                base: b.to_ascii_uppercase(),
                quote: q.to_ascii_uppercase(),
            });
        }
        if raw.len() >= 6 {
            let (b, q) = raw.split_at(raw.len() - 4);
            return Some(MarketSymbol {
                kind: MarketKind::Spot,
                base: b.to_ascii_uppercase(),
                quote: q.to_ascii_uppercase(),
            });
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct InvocationContext {
    pub market: Option<MarketSymbol>,
    pub venue_symbol: Option<String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct RestResponse {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub body: Bytes,
}
impl RestResponse {
    pub fn json_value(&self) -> Result<Value, InvokerError> {
        Ok(serde_json::from_slice(&self.body)?)
    }
    pub fn json_typed<T: DeserializeOwned>(&self) -> Result<T, InvokerError> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

#[derive(Debug, Clone)]
pub struct WsMessage {
    pub raw: Bytes,
    pub ts_recv: Instant,
}
impl WsMessage {
    pub fn json_value(&self) -> Result<Value, InvokerError> {
        Ok(serde_json::from_slice(&self.raw)?)
    }
}

#[derive(Clone)]
pub struct Invoker {
    client: reqwest::Client,
    config: Arc<InvokerConfig>,
}

impl Invoker {
    pub fn new(config: InvokerConfig) -> Result<Self, InvokerError> {
        let _ = SpecRegistry::global()?;
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .build()?;
        Ok(Self {
            client,
            config: Arc::new(config),
        })
    }
    pub fn list_venues(&self) -> Result<Vec<VenueId>, InvokerError> {
        Ok(SpecRegistry::global()?.list_venues())
    }
    pub fn list_ids(&self, venue: &VenueId) -> Result<Vec<OperationId>, InvokerError> {
        SpecRegistry::global()?.list_ids(venue)
    }

    pub async fn rest_call(
        &self,
        venue: &VenueId,
        id: &OperationId,
        mut ctx: InvocationContext,
    ) -> Result<RestResponse, InvokerError> {
        if ctx.venue_symbol.is_none() {
            if let Some(m) = ctx.market.as_ref() {
                ctx.venue_symbol = Some(SymbolCodec::encode(venue, m));
            }
        }
        self.rest_call_inner(venue, id, ctx).await
    }
    pub async fn rest_call_raw_symbol(
        &self,
        venue: &VenueId,
        id: &OperationId,
        venue_symbol: impl Into<String>,
        mut ctx: InvocationContext,
    ) -> Result<RestResponse, InvokerError> {
        ctx.venue_symbol = Some(venue_symbol.into());
        self.rest_call_inner(venue, id, ctx).await
    }

    async fn rest_call_inner(
        &self,
        venue: &VenueId,
        id: &OperationId,
        ctx: InvocationContext,
    ) -> Result<RestResponse, InvokerError> {
        let spec = SpecRegistry::global()?.resolve(venue, id)?;
        if spec.kind != OperationKind::Rest {
            return Err(InvokerError::KindMismatch {
                venue: venue.to_string(),
                id: id.to_string(),
                expected: "rest".into(),
                actual: "ws".into(),
            });
        }
        let method = spec.spec.method.clone().unwrap_or_else(|| "GET".into());
        let url = bind_template(
            &format!(
                "{}{}",
                spec.spec.base_url.clone().unwrap_or_default(),
                spec.spec.path.clone().unwrap_or_default()
            ),
            &ctx,
        )?;
        let retry_policy = RetryPolicy {
            base_delay_ms: self.config.base_backoff_ms,
            max_delay_ms: self.config.max_backoff_ms,
            jitter_ms: 0,
            respect_retry_after: true,
        };
        let mut attempt = 0;
        loop {
            let mut req = self
                .client
                .request(
                    reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                    &url,
                )
                .timeout(self.config.request_timeout);
            if !ctx.query.is_empty() {
                req = req.query(&ctx.query);
            }
            for (k, v) in &ctx.headers {
                req = req.header(k, v);
            }
            if let Some(body) = ctx.body.clone() {
                req = req.json(&body);
            }
            let resp = req.send().await?;
            if resp.status().as_u16() != 429 && !resp.status().is_server_error()
                || attempt >= self.config.max_retries
            {
                let status = resp.status().as_u16();
                let headers = resp.headers().clone();
                let body = resp.bytes().await?;
                return Ok(RestResponse {
                    status,
                    headers,
                    body,
                });
            }
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000);
            let mut delay = next_retry_delay_ms(&retry_policy, attempt, retry_after);
            delay += rand::thread_rng().gen_range(0..=20);
            sleep(Duration::from_millis(delay)).await;
            attempt += 1;
        }
    }

    pub async fn ws_subscribe(
        &self,
        venue: &VenueId,
        id: &OperationId,
        mut ctx: InvocationContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, InvokerError>> + Send>>, InvokerError>
    {
        if ctx.venue_symbol.is_none() {
            if let Some(m) = ctx.market.as_ref() {
                ctx.venue_symbol = Some(SymbolCodec::encode(venue, m));
            }
        }
        self.ws_subscribe_inner(venue, id, ctx).await
    }

    pub async fn ws_subscribe_raw_symbol(
        &self,
        venue: &VenueId,
        id: &OperationId,
        venue_symbol: impl Into<String>,
        mut ctx: InvocationContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, InvokerError>> + Send>>, InvokerError>
    {
        ctx.venue_symbol = Some(venue_symbol.into());
        self.ws_subscribe_inner(venue, id, ctx).await
    }

    async fn ws_subscribe_inner(
        &self,
        venue: &VenueId,
        id: &OperationId,
        ctx: InvocationContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, InvokerError>> + Send>>, InvokerError>
    {
        let spec = SpecRegistry::global()?.resolve(venue, id)?;
        if spec.kind != OperationKind::Ws {
            return Err(InvokerError::KindMismatch {
                venue: venue.to_string(),
                id: id.to_string(),
                expected: "ws".into(),
                actual: "rest".into(),
            });
        }
        let url = bind_template(&spec.ws_url()?, &ctx)?;
        let (tx, rx) = mpsc::channel(self.config.ws_buffer);
        let sub_tpl = spec
            .spec
            .channel
            .clone()
            .or_else(|| spec.spec.operation.clone())
            .unwrap_or_else(|| id.to_string());
        let sub_val = bind_template(&sub_tpl, &ctx)?;
        let max_reconnects = self.config.ws_max_reconnects;
        tokio::spawn(async move {
            let mut attempts = 0u32;
            loop {
                match tokio_tungstenite::connect_async(&url).await {
                    Ok((ws, _)) => {
                        let (mut w, mut r) = ws.split();
                        let payload = serde_json::json!({"op":"subscribe","channel":sub_val});
                        if w.send(Message::Text(payload.to_string())).await.is_err() {
                            attempts += 1;
                            if attempts > max_reconnects {
                                break;
                            }
                            continue;
                        }
                        let mut last = Instant::now();
                        loop {
                            let timeout = sleep(Duration::from_secs(30));
                            tokio::pin!(timeout);
                            tokio::select! {
                                _ = &mut timeout => { if last.elapsed()>Duration::from_secs(30) { break; } }
                                msg = r.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(t))) => { last=Instant::now(); if tx.send(Ok(WsMessage{raw:Bytes::from(t.into_bytes()), ts_recv:Instant::now()})).await.is_err(){return;} }
                                        Some(Ok(Message::Binary(b))) => { last=Instant::now(); if tx.send(Ok(WsMessage{raw:Bytes::from(b), ts_recv:Instant::now()})).await.is_err(){return;} }
                                        Some(Ok(Message::Ping(p))) => { let _=w.send(Message::Pong(p)).await; }
                                        Some(Ok(Message::Close(_))) | None => break,
                                        Some(Err(e)) => { let _=tx.send(Err(InvokerError::from(e))).await; break; }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(InvokerError::from(e))).await;
                    }
                }
                attempts += 1;
                if attempts > max_reconnects {
                    break;
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

impl Default for Invoker {
    fn default() -> Self {
        Self::new(InvokerConfig::default()).expect("invoker init")
    }
}

fn bind_template(input: &str, ctx: &InvocationContext) -> Result<String, InvokerError> {
    let mut out = input.to_string();
    let venue_symbol = ctx.venue_symbol.clone();
    let replacements = [
        ("symbol", venue_symbol.clone()),
        ("venue_symbol", venue_symbol.clone()),
        ("base", ctx.market.as_ref().map(|m| m.base.clone())),
        ("quote", ctx.market.as_ref().map(|m| m.quote.clone())),
    ];
    for (k, v) in replacements {
        if let Some(v) = v {
            out = out.replace(&format!("{{{k}}}"), &v);
        }
    }
    if out.contains('{') {
        return Err(InvokerError::MissingPlaceholder(out));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_symbol_parse_display() {
        let s: MarketSymbol = "SPOT:BTC/JPY".parse().unwrap();
        assert_eq!(s.to_string(), "SPOT:BTC/JPY");
        assert!("bad".parse::<MarketSymbol>().is_err());
    }

    #[test]
    fn venue_and_operation_validation() {
        assert!("binance-coinm".parse::<VenueId>().is_ok());
        assert!("x y".parse::<VenueId>().is_err());
        assert!("spot.public.rest.ticker".parse::<OperationId>().is_ok());
    }
}
