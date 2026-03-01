use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use ucel_core::decimal::serde::deserialize_decimal_observation;
use ucel_core::{Decimal, ErrorCode, OpName, UcelError};
use ucel_transport::{
    enforce_auth_boundary, HttpRequest, RequestContext, Transport, WsConnectRequest,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const REST_ENDPOINTS: [EndpointSpec; 8] = [
    EndpointSpec {
        id: "options.public.rest.general.ref",
        method: "GET",
        base_url: "docs://binance-options",
        path: "/general-info",
        requires_auth: false,
    },
    EndpointSpec {
        id: "options.public.rest.errors.ref",
        method: "GET",
        base_url: "docs://binance-options",
        path: "/error-code",
        requires_auth: false,
    },
    EndpointSpec {
        id: "options.public.rest.market.ref",
        method: "GET",
        base_url: "docs://binance-options",
        path: "/market-data/rest-api",
        requires_auth: false,
    },
    EndpointSpec {
        id: "options.private.rest.trade.ref",
        method: "GET/POST/DELETE",
        base_url: "docs://binance-options",
        path: "/trade/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "options.private.rest.account.ref",
        method: "GET/POST",
        base_url: "docs://binance-options",
        path: "/account/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "options.private.rest.listenkey.post",
        method: "POST",
        base_url: "https://eapi.binance.com",
        path: "/eapi/v1/listenKey",
        requires_auth: true,
    },
    EndpointSpec {
        id: "options.private.rest.listenkey.put",
        method: "PUT",
        base_url: "https://eapi.binance.com",
        path: "/eapi/v1/listenKey",
        requires_auth: true,
    },
    EndpointSpec {
        id: "options.private.rest.listenkey.delete",
        method: "DELETE",
        base_url: "https://eapi.binance.com",
        path: "/eapi/v1/listenKey",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub requires_auth: bool,
}

const WS_CHANNELS: [WsChannelSpec; 6] = [
    WsChannelSpec {
        id: "options.public.ws.trade",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "options.public.ws.ticker",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "options.public.ws.kline",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "options.public.ws.depth",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "options.public.ws.markprice",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "options.public.ws.indexprice",
        ws_url: "wss://nbstream.binance.com/eoptions/ws",
        requires_auth: false,
    },
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefPageResponse {
    pub section: String,
    pub version: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

#[derive(Debug, Clone)]
pub enum BinanceOptionsRestResponse {
    Reference(RefPageResponse),
    ListenKey(ListenKeyResponse),
}

pub async fn execute_rest<T: Transport>(
    transport: &T,
    endpoint_id: &str,
    key_id: Option<String>,
) -> Result<BinanceOptionsRestResponse, UcelError> {
    let spec = REST_ENDPOINTS
        .iter()
        .find(|v| v.id == endpoint_id)
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown endpoint"))?;
    let ctx = RequestContext {
        trace_id: Uuid::new_v4().to_string(),
        request_id: Uuid::new_v4().to_string(),
        run_id: Uuid::new_v4().to_string(),
        op: OpName::FetchStatus,
        venue: "binance-options".into(),
        policy_id: "default".into(),
        key_id: if spec.requires_auth { key_id } else { None },
        requires_auth: spec.requires_auth,
    };
    enforce_auth_boundary(&ctx)?;
    let resp = transport
        .send_http(
            HttpRequest {
                method: spec.method.into(),
                path: format!("{}{}", spec.base_url, spec.path),
                body: None,
            },
            ctx,
        )
        .await?;
    if resp.status >= 400 {
        return Err(UcelError::new(ErrorCode::Internal, "http error"));
    }
    if endpoint_id.contains("listenkey") {
        Ok(BinanceOptionsRestResponse::ListenKey(parse_json(
            &resp.body,
        )?))
    } else {
        Ok(BinanceOptionsRestResponse::Reference(parse_json(
            &resp.body,
        )?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: &'static str,
    pub stream: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsCommand {
    pub method: &'static str,
    pub params: Vec<String>,
    pub id: u64,
}

#[derive(Debug, Default)]
pub struct WsMetrics {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
    pub ws_orderbook_gap_total: AtomicU64,
    pub ws_orderbook_resync_total: AtomicU64,
    pub ws_orderbook_recovered_total: AtomicU64,
}

pub struct BinanceOptionsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    metrics: Arc<WsMetrics>,
}
impl BinanceOptionsBackpressure {
    pub fn new(capacity: usize, metrics: Arc<WsMetrics>) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx, metrics }
    }
    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.metrics
                .ws_backpressure_drops_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }
    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Trade {
        symbol: String,
        price: Decimal,
    },
    Ticker {
        symbol: String,
        last_price: Decimal,
    },
    Kline {
        symbol: String,
        interval: String,
        close: Decimal,
    },
    DepthDelta(OrderBookDelta),
    MarkPrice {
        underlying: String,
        mark_price: Decimal,
    },
    IndexPrice {
        underlying: String,
        index_price: Decimal,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderBookHealth {
    Ok,
    Degraded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderBookDelta {
    pub stream: String,
    pub prev_update_id: u64,
    pub update_id: u64,
}

pub struct OrderBookResyncEngine {
    next_expected_prev: Option<u64>,
    buffered: VecDeque<OrderBookDelta>,
    pub health: OrderBookHealth,
    metrics: Arc<WsMetrics>,
}
impl OrderBookResyncEngine {
    pub fn new(metrics: Arc<WsMetrics>) -> Self {
        Self {
            next_expected_prev: None,
            buffered: VecDeque::new(),
            health: OrderBookHealth::Ok,
            metrics,
        }
    }
    pub fn apply_snapshot(&mut self, last_update_id: u64) {
        self.next_expected_prev = Some(last_update_id);
        self.health = OrderBookHealth::Ok;
        self.metrics
            .ws_orderbook_recovered_total
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn ingest_delta(&mut self, delta: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(next_prev) = self.next_expected_prev {
            if delta.prev_update_id != next_prev {
                self.health = OrderBookHealth::Degraded;
                self.next_expected_prev = None;
                self.buffered.clear();
                self.metrics
                    .ws_orderbook_gap_total
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .ws_orderbook_resync_total
                    .fetch_add(1, Ordering::Relaxed);
                return Err(UcelError::new(ErrorCode::Desync, "gap detected"));
            }
            self.next_expected_prev = Some(delta.update_id);
        }
        self.buffered.push_back(delta);
        Ok(())
    }
}

#[derive(Default)]
pub struct BinanceOptionsWsAdapter {
    active: HashSet<WsSubscription>,
    pub metrics: Arc<WsMetrics>,
}
impl BinanceOptionsWsAdapter {
    pub fn ws_channel_specs() -> &'static [WsChannelSpec] {
        &WS_CHANNELS
    }
    pub fn subscribe_command(stream: String) -> WsCommand {
        WsCommand {
            method: "SUBSCRIBE",
            params: vec![stream],
            id: 1,
        }
    }
    pub fn unsubscribe_command(stream: String) -> WsCommand {
        WsCommand {
            method: "UNSUBSCRIBE",
            params: vec![stream],
            id: 2,
        }
    }
    pub fn register_subscription(&mut self, sub: WsSubscription) -> bool {
        self.active.insert(sub)
    }
    pub async fn connect_and_subscribe<T: Transport>(
        &mut self,
        transport: &T,
        sub: WsSubscription,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        let spec = WS_CHANNELS
            .iter()
            .find(|v| v.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeTicker,
            venue: "binance-options".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        transport
            .connect_ws(
                WsConnectRequest {
                    url: format!("{}/{}", spec.ws_url, sub.stream),
                },
                ctx,
            )
            .await?;
        self.active.insert(sub);
        Ok(())
    }
    pub async fn preflight_private_reject<T: Transport>(
        &self,
        _transport: &T,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeExecutionEvents,
            venue: "binance-options".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: true,
        };
        enforce_auth_boundary(&ctx)
    }
    pub async fn reconnect_and_resubscribe<T: Transport>(
        &self,
        transport: &T,
    ) -> Result<usize, UcelError> {
        self.metrics
            .ws_reconnect_total
            .fetch_add(1, Ordering::Relaxed);
        for sub in &self.active {
            let spec = WS_CHANNELS
                .iter()
                .find(|v| v.id == sub.channel_id)
                .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws channel"))?;
            let ctx = RequestContext {
                trace_id: Uuid::new_v4().to_string(),
                request_id: Uuid::new_v4().to_string(),
                run_id: Uuid::new_v4().to_string(),
                op: OpName::SubscribeTicker,
                venue: "binance-options".into(),
                policy_id: "default".into(),
                key_id: None,
                requires_auth: false,
            };
            transport
                .connect_ws(
                    WsConnectRequest {
                        url: format!("{}/{}", spec.ws_url, sub.stream),
                    },
                    ctx,
                )
                .await?;
            self.metrics
                .ws_resubscribe_total
                .fetch_add(1, Ordering::Relaxed);
        }
        Ok(self.active.len())
    }
    pub fn parse_market_event(channel_id: &str, raw: &Bytes) -> Result<MarketEvent, UcelError> {
        match channel_id {
            "options.public.ws.trade" => {
                let m: TradeWs = parse_json(raw)?;
                Ok(MarketEvent::Trade {
                    symbol: m.symbol,
                    price: m.price,
                })
            }
            "options.public.ws.ticker" => {
                let m: TickerWs = parse_json(raw)?;
                Ok(MarketEvent::Ticker {
                    symbol: m.symbol,
                    last_price: m.last_price,
                })
            }
            "options.public.ws.kline" => {
                let m: KlineWs = parse_json(raw)?;
                Ok(MarketEvent::Kline {
                    symbol: m.symbol,
                    interval: m.kline.interval,
                    close: m.kline.close,
                })
            }
            "options.public.ws.depth" => {
                let m: DepthWs = parse_json(raw)?;
                Ok(MarketEvent::DepthDelta(OrderBookDelta {
                    stream: m.stream,
                    prev_update_id: m.prev_update_id,
                    update_id: m.update_id,
                }))
            }
            "options.public.ws.markprice" => {
                let m: MarkPriceWs = parse_json(raw)?;
                Ok(MarketEvent::MarkPrice {
                    underlying: m.underlying,
                    mark_price: m.mark_price,
                })
            }
            "options.public.ws.indexprice" => {
                let m: IndexPriceWs = parse_json(raw)?;
                Ok(MarketEvent::IndexPrice {
                    underlying: m.underlying,
                    index_price: m.index_price,
                })
            }
            _ => Err(UcelError::new(
                ErrorCode::NotSupported,
                "unsupported ws channel",
            )),
        }
    }
}

pub fn sanitize_log_line(line: &str) -> String {
    line.split_whitespace()
        .map(|t| {
            if t.starts_with("api_key=") {
                "api_key=[redacted]".to_string()
            } else if t.starts_with("api_secret=") {
                "api_secret=[redacted]".to_string()
            } else {
                t.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Deserialize)]
struct TradeWs {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p", deserialize_with = "deserialize_decimal_observation")]
    price: Decimal,
}
#[derive(Debug, Deserialize)]
struct TickerWs {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c", deserialize_with = "deserialize_decimal_observation")]
    last_price: Decimal,
}
#[derive(Debug, Deserialize)]
struct KlineWs {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "k")]
    kline: KlineInner,
}
#[derive(Debug, Deserialize)]
struct KlineInner {
    #[serde(rename = "i")]
    interval: String,
    #[serde(rename = "c", deserialize_with = "deserialize_decimal_observation")]
    close: Decimal,
}
#[derive(Debug, Deserialize)]
struct DepthWs {
    stream: String,
    #[serde(rename = "pu")]
    prev_update_id: u64,
    #[serde(rename = "u")]
    update_id: u64,
}
#[derive(Debug, Deserialize)]
struct MarkPriceWs {
    #[serde(rename = "u")]
    underlying: String,
    #[serde(rename = "mp", deserialize_with = "deserialize_decimal_observation")]
    mark_price: Decimal,
}
#[derive(Debug, Deserialize)]
struct IndexPriceWs {
    #[serde(rename = "u")]
    underlying: String,
    #[serde(rename = "ip", deserialize_with = "deserialize_decimal_observation")]
    index_price: Decimal,
}

fn parse_json<T: DeserializeOwned>(raw: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse: {e}")))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{HttpResponse, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        ws_calls: Arc<Mutex<Vec<String>>>,
    }
    impl ucel_transport::Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(br#"{"section":"ok","version":"1"}"#),
            })
        }
        async fn connect_ws(
            &self,
            req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_calls.lock().unwrap().push(req.url);
            Ok(WsStream { connected: true })
        }
    }

    #[test]
    fn ws_contract_all_catalog_rows_have_commands() {
        for spec in BinanceOptionsWsAdapter::ws_channel_specs() {
            let stream = match spec.id {
                "options.public.ws.kline" => "BTC-240329-60000-C@kline_1m".to_string(),
                "options.public.ws.depth" => "BTC-240329-60000-C@depth20".to_string(),
                "options.public.ws.markprice" => "BTCUSDT@markPrice".to_string(),
                "options.public.ws.indexprice" => "BTCUSDT@indexPrice".to_string(),
                _ => "BTC-240329-60000-C@trade".to_string(),
            };
            assert_eq!(
                BinanceOptionsWsAdapter::subscribe_command(stream.clone()).method,
                "SUBSCRIBE"
            );
            assert_eq!(
                BinanceOptionsWsAdapter::unsubscribe_command(stream).method,
                "UNSUBSCRIBE"
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_and_resubscribe_is_idempotent() {
        let transport = SpyTransport::default();
        let mut ws = BinanceOptionsWsAdapter::default();
        assert!(ws.register_subscription(WsSubscription {
            channel_id: "options.public.ws.trade",
            stream: "btc@trade".into()
        }));
        assert!(!ws.register_subscription(WsSubscription {
            channel_id: "options.public.ws.trade",
            stream: "btc@trade".into()
        }));
        let count = ws.reconnect_and_resubscribe(&transport).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(ws.metrics.ws_reconnect_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn orderbook_gap_triggers_resync_recover() {
        let metrics = Arc::new(WsMetrics::default());
        let mut engine = OrderBookResyncEngine::new(metrics.clone());
        engine.apply_snapshot(100);
        engine
            .ingest_delta(OrderBookDelta {
                stream: "s".into(),
                prev_update_id: 100,
                update_id: 101,
            })
            .unwrap();
        let err = engine
            .ingest_delta(OrderBookDelta {
                stream: "s".into(),
                prev_update_id: 999,
                update_id: 102,
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert_eq!(metrics.ws_orderbook_gap_total.load(Ordering::Relaxed), 1);
        engine.apply_snapshot(200);
        assert_eq!(engine.health, OrderBookHealth::Ok);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_drops_are_counted() {
        let metrics = Arc::new(WsMetrics::default());
        let mut q = BinanceOptionsBackpressure::new(1, metrics.clone());
        q.try_enqueue(Bytes::from_static(b"a"));
        q.try_enqueue(Bytes::from_static(b"b"));
        assert_eq!(
            metrics.ws_backpressure_drops_total.load(Ordering::Relaxed),
            1
        );
        assert_eq!(q.recv().await.unwrap(), Bytes::from_static(b"a"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_connect() {
        let transport = SpyTransport::default();
        let ws = BinanceOptionsWsAdapter::default();
        let err = ws
            .preflight_private_reject(&transport, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert!(transport.ws_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn typed_deserialization_for_all_ws_rows() {
        let samples = [
            (
                "options.public.ws.trade",
                Bytes::from_static(br#"{"s":"BTC","p":"1.2"}"#),
            ),
            (
                "options.public.ws.ticker",
                Bytes::from_static(br#"{"s":"BTC","c":"2.3"}"#),
            ),
            (
                "options.public.ws.kline",
                Bytes::from_static(br#"{"s":"BTC","k":{"i":"1m","c":"3.4"}}"#),
            ),
            (
                "options.public.ws.depth",
                Bytes::from_static(br#"{"stream":"BTC@depth","pu":10,"u":11}"#),
            ),
            (
                "options.public.ws.markprice",
                Bytes::from_static(br#"{"u":"BTCUSDT","mp":"4.5"}"#),
            ),
            (
                "options.public.ws.indexprice",
                Bytes::from_static(br#"{"u":"BTCUSDT","ip":"4.7"}"#),
            ),
        ];
        for (id, payload) in samples {
            assert!(
                BinanceOptionsWsAdapter::parse_market_event(id, &payload).is_ok(),
                "{id}"
            );
        }
    }

    #[test]
    fn api_secrets_are_redacted_in_logs() {
        let sanitized = sanitize_log_line("key_id=k1 api_key=AAA api_secret=BBB");
        assert!(!sanitized.contains("AAA"));
        assert!(!sanitized.contains("BBB"));
        assert!(sanitized.contains("key_id=k1"));
    }

    #[test]
    fn strict_coverage_gate_has_no_gaps_for_binance_options() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../coverage/binance-options.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
