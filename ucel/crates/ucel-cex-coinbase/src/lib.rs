use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{
    Decimal, ErrorCode, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, Side, TradeEvent, UcelError,
};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub requires_auth: bool,
}

pub const REST_ENTRIES: [CatalogEntry; 7] = [
    CatalogEntry {
        id: "advanced.crypto.public.rest.reference.introduction",
        requires_auth: false,
    },
    CatalogEntry {
        id: "advanced.crypto.private.rest.reference.introduction",
        requires_auth: true,
    },
    CatalogEntry {
        id: "exchange.crypto.public.rest.reference.introduction",
        requires_auth: false,
    },
    CatalogEntry {
        id: "exchange.crypto.private.rest.reference.introduction",
        requires_auth: true,
    },
    CatalogEntry {
        id: "intx.crypto.public.rest.reference.welcome",
        requires_auth: false,
    },
    CatalogEntry {
        id: "intx.crypto.private.rest.reference.welcome",
        requires_auth: true,
    },
    CatalogEntry {
        id: "other.other.public.rest.docs.root",
        requires_auth: false,
    },
];

pub const WS_CHANNELS: [CatalogEntry; 8] = [
    CatalogEntry {
        id: "advanced.crypto.public.ws.reference.channels",
        requires_auth: false,
    },
    CatalogEntry {
        id: "advanced.crypto.private.ws.reference.guide",
        requires_auth: true,
    },
    CatalogEntry {
        id: "exchange.crypto.public.ws.reference.overview",
        requires_auth: false,
    },
    CatalogEntry {
        id: "exchange.crypto.private.ws.not_applicable.current_scope",
        requires_auth: true,
    },
    CatalogEntry {
        id: "intx.crypto.public.ws.reference.overview",
        requires_auth: false,
    },
    CatalogEntry {
        id: "intx.crypto.private.ws.reference.welcome",
        requires_auth: true,
    },
    CatalogEntry {
        id: "other.crypto.public.ws.common.protocol",
        requires_auth: false,
    },
    CatalogEntry {
        id: "other.other.public.ws.docs.root",
        requires_auth: false,
    },
];

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker {
        channel_id: String,
        symbol: String,
        price: f64,
    },
    Trades {
        channel_id: String,
        trades: Vec<TradeEvent>,
    },
    OrderbookSnapshot {
        channel_id: String,
        snapshot: OrderBookSnapshot,
    },
    Status {
        channel_id: String,
        status: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum CoinbaseWsMessage {
    #[serde(rename = "ticker")]
    Ticker {
        channel_id: String,
        symbol: String,
        price: f64,
    },
    #[serde(rename = "trades")]
    Trades {
        channel_id: String,
        trades: Vec<TradeWire>,
    },
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        channel_id: String,
        sequence: u64,
        bids: Vec<LevelWire>,
        asks: Vec<LevelWire>,
    },
    #[serde(rename = "status")]
    Status { channel_id: String, status: String },
}

#[derive(Debug, Clone, Deserialize)]
struct TradeWire {
    trade_id: String,
    price: Decimal,
    qty: Decimal,
    side: Side,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelWire {
    price: Decimal,
    qty: Decimal,
}

pub fn decode_market_event(raw: &[u8]) -> Result<MarketEvent, UcelError> {
    let msg: CoinbaseWsMessage = serde_json::from_slice(raw)
        .map_err(|e| UcelError::new(ErrorCode::WsProtocolViolation, e.to_string()))?;
    Ok(match msg {
        CoinbaseWsMessage::Ticker {
            channel_id,
            symbol,
            price,
        } => MarketEvent::Ticker {
            channel_id,
            symbol,
            price,
        },
        CoinbaseWsMessage::Trades { channel_id, trades } => MarketEvent::Trades {
            channel_id,
            trades: trades
                .into_iter()
                .map(|t| TradeEvent {
                    trade_id: t.trade_id,
                    price: t.price,
                    qty: t.qty,
                    side: t.side,
                })
                .collect(),
        },
        CoinbaseWsMessage::OrderbookSnapshot {
            channel_id,
            sequence,
            bids,
            asks,
        } => MarketEvent::OrderbookSnapshot {
            channel_id,
            snapshot: OrderBookSnapshot {
                bids: bids
                    .into_iter()
                    .map(|b| OrderBookLevel {
                        price: b.price,
                        qty: b.qty,
                    })
                    .collect(),
                asks: asks
                    .into_iter()
                    .map(|a| OrderBookLevel {
                        price: a.price,
                        qty: a.qty,
                    })
                    .collect(),
                sequence,
            },
        },
        CoinbaseWsMessage::Status { channel_id, status } => {
            MarketEvent::Status { channel_id, status }
        }
    })
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
}

pub struct CoinbaseBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}

impl CoinbaseBackpressure {
    pub fn new(capacity: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx, counters }
    }

    pub fn try_enqueue(&self, payload: Bytes) {
        if self.tx.try_send(payload).is_err() {
            self.counters
                .ws_backpressure_drops_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OrderbookHealth {
    #[default]
    Ok,
    Degraded,
}

#[derive(Debug, Default)]
pub struct OrderbookResync {
    buffered_deltas: VecDeque<OrderBookDelta>,
    next_sequence: Option<u64>,
    health: OrderbookHealth,
}

impl OrderbookResync {
    pub fn ingest_delta(&mut self, delta: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(expected) = self.next_sequence {
            if delta.sequence_start != expected {
                self.health = OrderbookHealth::Degraded;
                self.next_sequence = None;
                self.buffered_deltas.clear();
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "gap detected; force resync",
                ));
            }
            self.next_sequence = Some(delta.sequence_end + 1);
            return Ok(());
        }
        self.buffered_deltas.push_back(delta);
        Ok(())
    }

    pub fn apply_snapshot(
        &mut self,
        mut snapshot: OrderBookSnapshot,
    ) -> Result<OrderBookSnapshot, UcelError> {
        self.next_sequence = Some(snapshot.sequence + 1);
        while let Some(delta) = self.buffered_deltas.pop_front() {
            if delta.sequence_end <= snapshot.sequence {
                continue;
            }
            if delta.sequence_start > snapshot.sequence + 1 {
                self.health = OrderbookHealth::Degraded;
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "delta mismatch; force resync",
                ));
            }
            snapshot.sequence = delta.sequence_end;
        }
        self.health = OrderbookHealth::Ok;
        self.next_sequence = Some(snapshot.sequence + 1);
        Ok(snapshot)
    }

    pub fn health(&self) -> &OrderbookHealth {
        &self.health
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: &'static str,
    pub symbol: Option<String>,
}

pub struct CoinbaseWsClient {
    active: HashSet<WsSubscription>,
    counters: Arc<WsCounters>,
}

impl CoinbaseWsClient {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            active: HashSet::new(),
            counters,
        }
    }

    pub fn subscribe(
        &mut self,
        channel_id: &'static str,
        symbol: Option<String>,
        key_id: Option<String>,
    ) -> Result<bool, UcelError> {
        let spec = WS_CHANNELS
            .iter()
            .find(|c| c.id == channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "coinbase".into(),
            policy_id: "default".into(),
            key_id: key_id.clone(),
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        if spec.requires_auth {
            info!(target: "coinbase.auth", key_id = %key_id.as_deref().unwrap_or(""), "private ws subscribe preflight passed");
        }
        Ok(self.active.insert(WsSubscription { channel_id, symbol }))
    }

    pub fn unsubscribe(&mut self, channel_id: &'static str, symbol: Option<String>) {
        self.active.remove(&WsSubscription { channel_id, symbol });
    }

    pub fn reconnect_and_resubscribe(&self) -> Vec<WsSubscription> {
        self.counters
            .ws_reconnect_total
            .fetch_add(1, Ordering::Relaxed);
        self.counters
            .ws_resubscribe_total
            .fetch_add(self.active.len() as u64, Ordering::Relaxed);
        self.active.iter().cloned().collect()
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePayload {
    pub op: &'static str,
    pub channel: &'static str,
    pub symbol: Option<String>,
}

pub fn build_subscribe_payload(
    channel_id: &'static str,
    symbol: Option<String>,
) -> Result<SubscribePayload, UcelError> {
    WS_CHANNELS
        .iter()
        .find(|c| c.id == channel_id)
        .map(|c| SubscribePayload {
            op: "subscribe",
            channel: c.id,
            symbol,
        })
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))
}

#[derive(Debug, Clone)]
pub struct RestEndpointSpec {
    pub id: &'static str,
    pub requires_auth: bool,
    pub transport_enabled: bool,
}

static REST_ENDPOINT_SPECS: [RestEndpointSpec; 7] = [
    RestEndpointSpec { id: "advanced.crypto.public.rest.reference.introduction", requires_auth: false, transport_enabled: true },
    RestEndpointSpec { id: "advanced.crypto.private.rest.reference.introduction", requires_auth: true, transport_enabled: true },
    RestEndpointSpec { id: "exchange.crypto.public.rest.reference.introduction", requires_auth: false, transport_enabled: true },
    RestEndpointSpec { id: "exchange.crypto.private.rest.reference.introduction", requires_auth: true, transport_enabled: true },
    RestEndpointSpec { id: "intx.crypto.public.rest.reference.welcome", requires_auth: false, transport_enabled: true },
    RestEndpointSpec { id: "intx.crypto.private.rest.reference.welcome", requires_auth: true, transport_enabled: true },
    RestEndpointSpec { id: "other.other.public.rest.docs.root", requires_auth: false, transport_enabled: true },
];

#[derive(Debug, Clone, Deserialize)]
pub struct RestReferenceBody {
    pub id: String,
    pub source: String,
}

#[derive(Debug)]
pub enum CoinbaseRestResponse {
    Reference(RestReferenceBody),
    ReferenceOnly(RestReferenceBody),
}

#[derive(Default)]
pub struct CoinbaseRestAdapter;

impl CoinbaseRestAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn endpoint_specs() -> &'static [RestEndpointSpec] {
        &REST_ENDPOINT_SPECS
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        id: &str,
        _symbol: Option<String>,
        key_id: Option<String>,
    ) -> Result<CoinbaseRestResponse, UcelError> {
        let spec = REST_ENDPOINT_SPECS
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown endpoint"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "coinbase".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;
        let resp = transport
            .send_http(
                HttpRequest {
                    path: format!("/rest/{id}"),
                    method: "GET".into(),
                    body: None,
                },
                ctx,
            )
            .await?;
        if resp.status == 429 {
            let retry_after_ms = serde_json::from_slice::<serde_json::Value>(&resp.body)
                .ok()
                .and_then(|v| v["retry_after_ms"].as_u64());
            return Err(UcelError::new(ErrorCode::RateLimited, "rate limited")
                .with_retry_after_ms(retry_after_ms.unwrap_or(0)));
        }
        if resp.status >= 500 {
            return Err(UcelError::new(ErrorCode::Upstream5xx, "upstream error"));
        }
        let body: RestReferenceBody = serde_json::from_slice(&resp.body)
            .map_err(|e| UcelError::new(ErrorCode::Internal, e.to_string()))?;
        Ok(CoinbaseRestResponse::Reference(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};

    #[test]
    fn strict_coverage_gate_for_coinbase_is_zero_gap() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/coinbase.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict, "coinbase strict gate must be enabled");
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }

    #[test]
    fn subscribe_payload_builds_for_all_catalog_ws_rows() {
        for spec in WS_CHANNELS {
            let payload = build_subscribe_payload(spec.id, Some("BTC-USD".into())).unwrap();
            assert_eq!(payload.channel, spec.id);
        }
    }

    #[test]
    fn typed_deserialize_and_normalize_market_events() {
        let ticker = br#"{"type":"ticker","channel_id":"advanced.crypto.public.ws.reference.channels","symbol":"BTC-USD","price":100.0}"#;
        let trades = br#"{"type":"trades","channel_id":"exchange.crypto.public.ws.reference.overview","trades":[{"trade_id":"t1","price":1.0,"qty":2.0,"side":"buy"}]}"#;
        let _ = decode_market_event(ticker).unwrap();
        let _ = decode_market_event(trades).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_is_idempotent() {
        let counters = Arc::new(WsCounters::default());
        let mut client = CoinbaseWsClient::new(counters.clone());
        assert!(client
            .subscribe(WS_CHANNELS[0].id, Some("BTC-USD".into()), None)
            .unwrap());
        assert!(!client
            .subscribe(WS_CHANNELS[0].id, Some("BTC-USD".into()), None)
            .unwrap());
        let recovered = client.reconnect_and_resubscribe();
        assert_eq!(recovered.len(), 1);
        assert_eq!(client.active_len(), 1);
        assert_eq!(counters.ws_reconnect_total.load(Ordering::Relaxed), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_bounded_channel_counts_drop() {
        let counters = Arc::new(WsCounters::default());
        let mut queue = CoinbaseBackpressure::new(1, counters.clone());
        queue.try_enqueue(Bytes::from_static(b"a"));
        queue.try_enqueue(Bytes::from_static(b"b"));
        assert_eq!(
            counters.ws_backpressure_drops_total.load(Ordering::Relaxed),
            1
        );
        assert_eq!(queue.recv().await.unwrap(), Bytes::from_static(b"a"));
    }

    #[test]
    fn orderbook_gap_forces_resync_then_recover() {
        let mut engine = OrderbookResync::default();
        engine
            .ingest_delta(OrderBookDelta {
                bids: vec![],
                asks: vec![],
                sequence_start: 11,
                sequence_end: 11,
            })
            .unwrap();
        let snapshot = engine
            .apply_snapshot(OrderBookSnapshot {
                bids: vec![],
                asks: vec![],
                sequence: 10,
            })
            .unwrap();
        assert_eq!(snapshot.sequence, 11);
        let err = engine
            .ingest_delta(OrderBookDelta {
                bids: vec![],
                asks: vec![],
                sequence_start: 13,
                sequence_end: 13,
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert_eq!(engine.health(), &OrderbookHealth::Degraded);
        let recovered = engine
            .apply_snapshot(OrderBookSnapshot {
                bids: vec![],
                asks: vec![],
                sequence: 13,
            })
            .unwrap();
        assert_eq!(recovered.sequence, 13);
        assert_eq!(engine.health(), &OrderbookHealth::Ok);
    }

    #[test]
    fn duplicate_or_out_of_order_policy_is_safe_resync() {
        let mut engine = OrderbookResync {
            next_sequence: Some(5),
            ..Default::default()
        };
        let err = engine
            .ingest_delta(OrderBookDelta {
                bids: vec![],
                asks: vec![],
                sequence_start: 4,
                sequence_end: 4,
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
    }

    #[test]
    fn private_preflight_rejects_without_key() {
        let counters = Arc::new(WsCounters::default());
        let mut client = CoinbaseWsClient::new(counters);
        let err = client.subscribe(
            "advanced.crypto.private.ws.reference.guide",
            Some("BTC-USD".into()),
            None,
        );
        assert!(err.is_err());
        assert_eq!(client.active_len(), 0);
    }

    #[test]
    fn tracing_log_never_contains_secret_material() {
        #[derive(Clone, Default)]
        struct SharedWriter(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for SharedWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let sink = SharedWriter::default();
        let captured = sink.0.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || sink.clone())
            .without_time()
            .finish();

        let counters = Arc::new(WsCounters::default());
        let mut client = CoinbaseWsClient::new(counters);
        tracing::subscriber::with_default(subscriber, || {
            let _ = client.subscribe(
                "advanced.crypto.private.ws.reference.guide",
                None,
                Some("key-123".into()),
            );
        });

        let logs = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
        assert!(logs.contains("key-123"));
        assert!(!logs.contains("api_secret"));
        assert!(!logs.contains("secret"));
    }
}

pub mod channels;
pub mod symbols;
pub mod ws_manager;
