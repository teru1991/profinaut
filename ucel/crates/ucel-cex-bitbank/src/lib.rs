use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use ucel_core::{
    ErrorCode, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError,
};
use ucel_transport::{enforce_auth_boundary, RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    ws_channels: Vec<WsSpec>,
    rest_endpoints: Vec<RestSpec>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WsSpec {
    pub id: String,
    pub ws_url: Option<String>,
    pub channel: Option<String>,
    pub visibility: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct RestSpec {
    pub id: String,
    pub visibility: String,
}

pub fn rest_specs() -> Vec<RestSpec> {
    serde_json::from_str::<Catalog>(include_str!(
        "../../../../docs/exchanges/bitbank/catalog.json"
    ))
    .expect("catalog")
    .rest_endpoints
}
pub fn ws_specs() -> Vec<WsSpec> {
    serde_json::from_str::<Catalog>(include_str!(
        "../../../../docs/exchanges/bitbank/catalog.json"
    ))
    .expect("catalog")
    .ws_channels
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
}

pub struct BitbankWsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}
impl BitbankWsBackpressure {
    pub fn new(cap: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx, counters }
    }
    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.counters
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
    Ticker {
        pair: String,
        last: f64,
    },
    Trade(TradeEvent),
    OrderBookDelta(OrderBookDelta),
    OrderBookSnapshot(OrderBookSnapshot),
    CircuitBreak {
        pair: String,
        mode: String,
    },
    UserEvent {
        kind: String,
        fields: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub pair: Option<String>,
}

pub struct BitbankWsAdapter {
    counters: Arc<WsCounters>,
    active: HashSet<WsSubscription>,
}
impl BitbankWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            counters,
            active: HashSet::new(),
        }
    }
    pub fn subscribe_command(&self, id: &str, pair: Option<&str>) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        let channel = spec
            .channel
            .unwrap_or_default()
            .replace("{pair}", pair.unwrap_or("btc_jpy"));
        Ok(format!(r#"42[\"join-room\",\"{channel}\"]"#))
    }
    pub fn unsubscribe_command(&self, id: &str, pair: Option<&str>) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        let channel = spec
            .channel
            .unwrap_or_default()
            .replace("{pair}", pair.unwrap_or("btc_jpy"));
        Ok(format!(r#"42[\"leave-room\",\"{channel}\"]"#))
    }
    pub async fn connect_and_subscribe<T: Transport>(
        &mut self,
        t: &T,
        sub: WsSubscription,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::SubscribeTicker,
            venue: "bitbank".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.visibility == "private",
        };
        enforce_auth_boundary(&ctx)?;
        t.connect_ws(
            WsConnectRequest {
                url: spec.ws_url.unwrap_or_else(|| "wss://private-stream".into()),
            },
            ctx,
        )
        .await?;
        self.active.insert(sub);
        Ok(())
    }
    pub async fn reconnect_and_resubscribe<T: Transport>(
        &self,
        t: &T,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        self.counters
            .ws_reconnect_total
            .fetch_add(1, Ordering::Relaxed);
        for s in &self.active {
            let mut clone = Self {
                counters: self.counters.clone(),
                active: HashSet::new(),
            };
            clone
                .connect_and_subscribe(t, s.clone(), key_id.clone())
                .await?;
            self.counters
                .ws_resubscribe_total
                .fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
    pub fn parse_market_event(&self, id: &str, body: &Bytes) -> Result<MarketEvent, UcelError> {
        match id {
            "crypto.public.ws.market.ticker" => {
                let v: TickerMsg = parse_json(body)?;
                Ok(MarketEvent::Ticker {
                    pair: v.pair,
                    last: num(&v.last)?,
                })
            }
            "crypto.public.ws.market.transactions" => {
                let v: TradeMsg = parse_json(body)?;
                Ok(MarketEvent::Trade(TradeEvent {
                    trade_id: v.transaction_id,
                    price: num(&v.price)?,
                    qty: num(&v.amount)?,
                    side: v.side,
                }))
            }
            "crypto.public.ws.market.depth-diff" => {
                let v: DepthMsg = parse_json(body)?;
                Ok(MarketEvent::OrderBookDelta(OrderBookDelta {
                    bids: levels(v.bids)?,
                    asks: levels(v.asks)?,
                    sequence_start: v.sequence,
                    sequence_end: v.sequence,
                }))
            }
            "crypto.public.ws.market.depth-whole" => {
                let v: DepthMsg = parse_json(body)?;
                Ok(MarketEvent::OrderBookSnapshot(OrderBookSnapshot {
                    bids: levels(v.bids)?,
                    asks: levels(v.asks)?,
                    sequence: v.sequence,
                }))
            }
            "crypto.public.ws.market.circuit-break-info" => {
                let v: CircuitMsg = parse_json(body)?;
                Ok(MarketEvent::CircuitBreak {
                    pair: v.pair,
                    mode: v.mode,
                })
            }
            _ => {
                let v: UserMsg = parse_json(body)?;
                Ok(MarketEvent::UserEvent {
                    kind: v.event,
                    fields: v.fields,
                })
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct OrderbookResync {
    pub degraded: bool,
    next: Option<u64>,
    pending: VecDeque<OrderBookDelta>,
}
impl OrderbookResync {
    pub fn on_delta(&mut self, d: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(next) = self.next {
            if d.sequence_start != next {
                self.degraded = true;
                self.next = None;
                self.pending.clear();
                return Err(UcelError::new(ErrorCode::Desync, "gap"));
            }
            self.next = Some(d.sequence_end + 1);
        } else {
            self.pending.push_back(d);
        }
        Ok(())
    }
    pub fn apply_snapshot(&mut self, mut s: OrderBookSnapshot) -> OrderBookSnapshot {
        self.next = Some(s.sequence + 1);
        while let Some(d) = self.pending.pop_front() {
            s.sequence = d.sequence_end;
        }
        self.degraded = false;
        s
    }
}

#[derive(Deserialize)]
struct TickerMsg {
    pair: String,
    last: String,
}
#[derive(Deserialize)]
struct TradeMsg {
    transaction_id: String,
    price: String,
    amount: String,
    side: String,
}
#[derive(Deserialize)]
struct DepthMsg {
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
    sequence: u64,
}
#[derive(Deserialize)]
struct CircuitMsg {
    pair: String,
    mode: String,
}
#[derive(Deserialize)]
struct UserMsg {
    event: String,
    #[serde(flatten)]
    fields: BTreeMap<String, String>,
}

fn parse_json<T: DeserializeOwned>(b: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(b).map_err(|e| UcelError::new(ErrorCode::Internal, format!("json: {e}")))
}
fn num(v: &str) -> Result<f64, UcelError> {
    v.parse()
        .map_err(|_| UcelError::new(ErrorCode::Internal, "num"))
}
fn levels(v: Vec<[String; 2]>) -> Result<Vec<OrderBookLevel>, UcelError> {
    v.into_iter()
        .map(|x| {
            Ok(OrderBookLevel {
                price: num(&x[0])?,
                qty: num(&x[1])?,
            })
        })
        .collect()
}

pub fn sanitize_log_line(line: &str) -> String {
    line.split_whitespace()
        .map(|token| {
            if token.starts_with("api_key=") {
                "api_key=[redacted]".to_string()
            } else if token.starts_with("api_secret=") {
                "api_secret=[redacted]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as O};
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{HttpRequest, HttpResponse, WsStream};

    struct Spy {
        ws: AtomicUsize,
    }
    impl Spy {
        fn new() -> Self {
            Self {
                ws: AtomicUsize::new(0),
            }
        }
    }
    impl Transport for Spy {
        async fn send_http(
            &self,
            _: HttpRequest,
            _: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Err(UcelError::new(ErrorCode::NotSupported, "na"))
        }
        async fn connect_ws(
            &self,
            _: WsConnectRequest,
            _: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws.fetch_add(1, O::SeqCst);
            Ok(WsStream::default())
        }
    }

    #[test]
    fn ws_contract_all_channels_tested() {
        let ws = BitbankWsAdapter::new(Arc::new(WsCounters::default()));
        for s in ws_specs() {
            let _ = ws.subscribe_command(&s.id, Some("btc_jpy")).unwrap();
            let _ = ws.unsubscribe_command(&s.id, Some("btc_jpy")).unwrap();
        }
    }
    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_and_private_preflight() {
        let mut ws = BitbankWsAdapter::new(Arc::new(WsCounters::default()));
        let spy = Spy::new();
        let e = ws
            .connect_and_subscribe(
                &spy,
                WsSubscription {
                    channel_id: "crypto.private.ws.user.stream.deposit".into(),
                    pair: None,
                },
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::MissingAuth);
        ws.connect_and_subscribe(
            &spy,
            WsSubscription {
                channel_id: "crypto.public.ws.market.ticker".into(),
                pair: Some("btc_jpy".into()),
            },
            None,
        )
        .await
        .unwrap();
        ws.reconnect_and_resubscribe(&spy, None).await.unwrap();
    }
    #[test]
    fn orderbook_gap_resync_and_parse() {
        let mut r = OrderbookResync::default();
        r.on_delta(OrderBookDelta {
            bids: vec![],
            asks: vec![],
            sequence_start: 11,
            sequence_end: 11,
        })
        .unwrap();
        assert_eq!(
            r.apply_snapshot(OrderBookSnapshot {
                bids: vec![],
                asks: vec![],
                sequence: 10
            })
            .sequence,
            11
        );
        assert!(r
            .on_delta(OrderBookDelta {
                bids: vec![],
                asks: vec![],
                sequence_start: 13,
                sequence_end: 13
            })
            .is_err());
        let ws = BitbankWsAdapter::new(Arc::new(WsCounters::default()));
        assert!(matches!(
            ws.parse_market_event(
                "crypto.public.ws.market.depth-whole",
                &Bytes::from_static(br#"{"bids":[["1","2"]],"asks":[["2","1"]],"sequence":1}"#)
            )
            .unwrap(),
            MarketEvent::OrderBookSnapshot(_)
        ));
    }
    #[tokio::test(flavor = "current_thread")]
    async fn backpressure_and_secret_and_strict_gate() {
        let counters = Arc::new(WsCounters::default());
        let mut q = BitbankWsBackpressure::new(1, counters.clone());
        q.try_enqueue(Bytes::from_static(b"1"));
        q.try_enqueue(Bytes::from_static(b"2"));
        assert_eq!(counters.ws_backpressure_drops_total.load(O::Relaxed), 1);
        assert_eq!(q.recv().await.unwrap(), Bytes::from_static(b"1"));
        let out = sanitize_log_line("api_key=aaa api_secret=bbb key_id=k1");
        assert!(!out.contains("aaa") && !out.contains("bbb") && out.contains("key_id=k1"));
        let m = load_coverage_manifest(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitbank.yaml"),
        )
        .unwrap();
        assert!(m.strict);
        assert!(evaluate_coverage_gate(&m).is_empty());
    }
}
