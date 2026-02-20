use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{
    ErrorCode, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError,
};
use ucel_transport::{enforce_auth_boundary, RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    ws_channels: Vec<WsSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsSpec {
    pub id: String,
    pub access: String,
    pub ws_url: String,
    pub channel: String,
}

pub fn ws_specs() -> Vec<WsSpec> {
    serde_json::from_str::<Catalog>(include_str!(
        "../../../../docs/exchanges/bittrade/catalog.json"
    ))
    .expect("bittrade catalog")
    .ws_channels
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
}

pub struct BittradeWsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}
impl BittradeWsBackpressure {
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
    Kline {
        ch: String,
        close: f64,
    },
    OrderBookDelta(OrderBookDelta),
    Bbo {
        ch: String,
        bid: f64,
        ask: f64,
    },
    Ticker {
        ch: String,
        close: f64,
    },
    Trade(TradeEvent),
    AccountUpdate {
        ch: String,
        count: usize,
    },
    TradeClearing {
        ch: String,
        fields: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub symbol: String,
    pub period_or_type: Option<String>,
}

pub struct BittradeWsAdapter {
    counters: Arc<WsCounters>,
    active: HashSet<WsSubscription>,
}

impl BittradeWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            counters,
            active: HashSet::new(),
        }
    }

    pub fn subscribe_command(&self, sub: &WsSubscription) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        Ok(format!(
            r#"{{\"sub\":\"{}\",\"id\":\"{}\"}}"#,
            render_channel(&spec.channel, sub),
            sub.symbol
        ))
    }

    pub fn unsubscribe_command(&self, sub: &WsSubscription) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        Ok(format!(
            r#"{{\"unsub\":\"{}\",\"id\":\"{}\"}}"#,
            render_channel(&spec.channel, sub),
            sub.symbol
        ))
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
            venue: "bittrade".into(),
            policy_id: "default".into(),
            key_id: key_id.clone(),
            requires_auth: spec.access == "private",
        };
        enforce_auth_boundary(&ctx)?;
        info!(venue="bittrade", key_id=?key_id, channel_id=%sub.channel_id, "ws subscribe preflight passed");
        t.connect_ws(WsConnectRequest { url: spec.ws_url }, ctx)
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
            let mut replay = Self::new(self.counters.clone());
            replay
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
            "public.ws.market.kline" => {
                let m: KlineMsg = parse_json(body)?;
                Ok(MarketEvent::Kline {
                    ch: m.ch,
                    close: m.tick.close,
                })
            }
            "public.ws.market.depth" => {
                let m: DepthMsg = parse_json(body)?;
                Ok(MarketEvent::OrderBookDelta(OrderBookDelta {
                    bids: levels(m.tick.bids)?,
                    asks: levels(m.tick.asks)?,
                    sequence_start: m.tick.version,
                    sequence_end: m.tick.version,
                }))
            }
            "public.ws.market.bbo" => {
                let m: BboMsg = parse_json(body)?;
                Ok(MarketEvent::Bbo {
                    ch: m.ch,
                    bid: m.tick.bid,
                    ask: m.tick.ask,
                })
            }
            "public.ws.market.detail" => {
                let m: DetailMsg = parse_json(body)?;
                Ok(MarketEvent::Ticker {
                    ch: m.ch,
                    close: m.tick.close,
                })
            }
            "public.ws.market.trade.detail" => {
                let m: TradeMsg = parse_json(body)?;
                let t = m.tick.data.into_iter().next().ok_or_else(|| {
                    UcelError::new(ErrorCode::CatalogMissingField, "trade data empty")
                })?;
                Ok(MarketEvent::Trade(TradeEvent {
                    trade_id: t.id,
                    price: t.price,
                    qty: t.amount,
                    side: t.direction,
                }))
            }
            "private.ws.accounts.update" => {
                let m: AccountsMsg = parse_json(body)?;
                Ok(MarketEvent::AccountUpdate {
                    ch: m.ch,
                    count: m.data.len(),
                })
            }
            "private.ws.trade.clearing" => {
                let m: ClearingMsg = parse_json(body)?;
                Ok(MarketEvent::TradeClearing {
                    ch: m.ch,
                    fields: m.data,
                })
            }
            _ => Err(UcelError::new(
                ErrorCode::NotSupported,
                "unknown channel id",
            )),
        }
    }
}

fn render_channel(template: &str, sub: &WsSubscription) -> String {
    template
        .replace("$symbol", &sub.symbol)
        .replace("$period", sub.period_or_type.as_deref().unwrap_or("1min"))
        .replace("$type", sub.period_or_type.as_deref().unwrap_or("step0"))
}

#[derive(Debug, Default)]
pub struct OrderbookResync {
    pub degraded: bool,
    next: Option<u64>,
    pending: VecDeque<OrderBookDelta>,
}
impl OrderbookResync {
    pub fn on_snapshot(&mut self, snap: OrderBookSnapshot) -> Vec<OrderBookDelta> {
        self.degraded = false;
        self.next = Some(snap.sequence + 1);
        let mut applied = Vec::new();
        while let Some(front) = self.pending.front() {
            if front.sequence_end < snap.sequence + 1 {
                self.pending.pop_front();
                continue;
            }
            if front.sequence_start == self.next.unwrap_or_default() {
                let d = self.pending.pop_front().expect("front exists");
                self.next = Some(d.sequence_end + 1);
                applied.push(d);
                continue;
            }
            break;
        }
        applied
    }

    pub fn on_delta(&mut self, d: OrderBookDelta) -> Result<(), UcelError> {
        if let Some(next) = self.next {
            if d.sequence_end < next {
                return Ok(());
            }
            if d.sequence_start != next {
                self.degraded = true;
                self.next = None;
                self.pending.clear();
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "gap detected, resync required",
                ));
            }
            self.next = Some(d.sequence_end + 1);
            return Ok(());
        }
        self.pending.push_back(d);
        Ok(())
    }
}

fn parse_json<T: DeserializeOwned>(body: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(body).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("ws payload parse failed: {e}"),
        )
    })
}

fn levels(raw: Vec<[f64; 2]>) -> Result<Vec<OrderBookLevel>, UcelError> {
    if raw.is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "levels empty",
        ));
    }
    Ok(raw
        .into_iter()
        .map(|lv| OrderBookLevel {
            price: lv[0],
            qty: lv[1],
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct KlineMsg {
    ch: String,
    tick: KlineTick,
}
#[derive(Debug, Deserialize)]
struct KlineTick {
    close: f64,
}

#[derive(Debug, Deserialize)]
struct DepthMsg {
    tick: DepthTick,
}
#[derive(Debug, Deserialize)]
struct DepthTick {
    bids: Vec<[f64; 2]>,
    asks: Vec<[f64; 2]>,
    version: u64,
}

#[derive(Debug, Deserialize)]
struct BboMsg {
    ch: String,
    tick: BboTick,
}
#[derive(Debug, Deserialize)]
struct BboTick {
    bid: f64,
    ask: f64,
}

#[derive(Debug, Deserialize)]
struct DetailMsg {
    ch: String,
    tick: DetailTick,
}
#[derive(Debug, Deserialize)]
struct DetailTick {
    close: f64,
}

#[derive(Debug, Deserialize)]
struct TradeMsg {
    tick: TradeTick,
}
#[derive(Debug, Deserialize)]
struct TradeTick {
    data: Vec<TradeRow>,
}
#[derive(Debug, Deserialize)]
struct TradeRow {
    id: String,
    price: f64,
    amount: f64,
    direction: String,
}

#[derive(Debug, Deserialize)]
struct AccountsMsg {
    ch: String,
    data: Vec<BTreeMap<String, String>>,
}
#[derive(Debug, Deserialize)]
struct ClearingMsg {
    ch: String,
    data: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use ucel_testkit::{
        load_coverage_manifest, run_coverage_gate, CatalogContractIndex, CoverageGateResult,
    };
    use ucel_transport::{HttpRequest, HttpResponse, Transport, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        ws_connects: Arc<Mutex<Vec<String>>>,
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            unreachable!()
        }

        async fn connect_ws(
            &self,
            req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_connects.lock().unwrap().push(req.url);
            Ok(WsStream { connected: true })
        }
    }

    #[test]
    fn all_catalog_ws_channels_have_contract_tests() {
        let mut ix = CatalogContractIndex::default();
        for spec in ws_specs() {
            ix.register_id(&spec.id);
        }
        let catalog: Catalog = serde_json::from_str(include_str!(
            "../../../../docs/exchanges/bittrade/catalog.json"
        ))
        .unwrap();
        let reg = ucel_registry::ExchangeCatalog {
            exchange: "bittrade".into(),
            rest_endpoints: vec![],
            ws_channels: catalog
                .ws_channels
                .into_iter()
                .map(|w| ucel_registry::CatalogEntry {
                    id: w.id,
                    visibility: None,
                    access: w.access,
                    requires_auth: None,
                    operation: None,
                    method: None,
                    base_url: None,
                    path: None,
                    ws_url: Some(w.ws_url),
                    ws: None,
                    auth: ucel_registry::CatalogAuth::default(),
                })
                .collect(),
        };
        assert!(ix.missing_catalog_ids(&reg).is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_and_idempotent() {
        let mut adapter = BittradeWsAdapter::new(Arc::new(WsCounters::default()));
        let t = SpyTransport::default();
        let sub = WsSubscription {
            channel_id: "public.ws.market.kline".into(),
            symbol: "btcjpy".into(),
            period_or_type: Some("1min".into()),
        };
        adapter
            .connect_and_subscribe(&t, sub.clone(), None)
            .await
            .unwrap();
        adapter
            .connect_and_subscribe(&t, sub.clone(), None)
            .await
            .unwrap();
        assert_eq!(adapter.active.len(), 1);
        adapter.reconnect_and_resubscribe(&t, None).await.unwrap();
        assert_eq!(
            adapter.counters.ws_reconnect_total.load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            adapter
                .counters
                .ws_resubscribe_total
                .load(Ordering::Relaxed),
            1
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_reject_blocks_connect() {
        let mut adapter = BittradeWsAdapter::new(Arc::new(WsCounters::default()));
        let t = SpyTransport::default();
        let err = adapter
            .connect_and_subscribe(
                &t,
                WsSubscription {
                    channel_id: "private.ws.accounts.update".into(),
                    symbol: "accounts.update".into(),
                    period_or_type: None,
                },
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(t.ws_connects.lock().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_drops_are_counted() {
        let mut q = BittradeWsBackpressure::new(1, Arc::new(WsCounters::default()));
        q.try_enqueue(Bytes::from_static(b"a"));
        q.try_enqueue(Bytes::from_static(b"b"));
        assert_eq!(
            q.counters
                .ws_backpressure_drops_total
                .load(Ordering::Relaxed),
            1
        );
        assert_eq!(q.recv().await.unwrap(), Bytes::from_static(b"a"));
    }

    #[test]
    fn orderbook_gap_duplicate_and_resync_recovered() {
        let mut sync = OrderbookResync::default();
        assert!(sync
            .on_delta(OrderBookDelta {
                bids: vec![OrderBookLevel {
                    price: 1.0,
                    qty: 1.0
                }],
                asks: vec![OrderBookLevel {
                    price: 2.0,
                    qty: 1.0
                }],
                sequence_start: 9,
                sequence_end: 9
            })
            .is_ok());
        let applied = sync.on_snapshot(OrderBookSnapshot {
            bids: vec![OrderBookLevel {
                price: 1.0,
                qty: 1.0,
            }],
            asks: vec![OrderBookLevel {
                price: 2.0,
                qty: 1.0,
            }],
            sequence: 9,
        });
        assert_eq!(applied.len(), 0);
        sync.on_delta(OrderBookDelta {
            bids: vec![OrderBookLevel {
                price: 1.1,
                qty: 1.0,
            }],
            asks: vec![OrderBookLevel {
                price: 2.1,
                qty: 1.0,
            }],
            sequence_start: 10,
            sequence_end: 10,
        })
        .unwrap();
        assert!(sync
            .on_delta(OrderBookDelta {
                bids: vec![OrderBookLevel {
                    price: 1.1,
                    qty: 1.0
                }],
                asks: vec![OrderBookLevel {
                    price: 2.1,
                    qty: 1.0
                }],
                sequence_start: 10,
                sequence_end: 10
            })
            .is_ok());
        let err = sync
            .on_delta(OrderBookDelta {
                bids: vec![OrderBookLevel {
                    price: 1.2,
                    qty: 1.0,
                }],
                asks: vec![OrderBookLevel {
                    price: 2.2,
                    qty: 1.0,
                }],
                sequence_start: 12,
                sequence_end: 12,
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Desync);
        assert!(sync.degraded);
        let recovered = sync.on_snapshot(OrderBookSnapshot {
            bids: vec![OrderBookLevel {
                price: 1.2,
                qty: 1.0,
            }],
            asks: vec![OrderBookLevel {
                price: 2.2,
                qty: 1.0,
            }],
            sequence: 12,
        });
        assert!(recovered.is_empty());
        assert!(!sync.degraded);
    }

    #[test]
    fn parse_all_ws_channels_typed() {
        let adapter = BittradeWsAdapter::new(Arc::new(WsCounters::default()));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "public.ws.market.kline",
                    &Bytes::from_static(
                        br#"{"ch":"market.btcjpy.kline.1min","tick":{"close":1.23}}"#
                    )
                )
                .unwrap(),
            MarketEvent::Kline { .. }
        ));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "public.ws.market.depth",
                    &Bytes::from_static(
                        br#"{"tick":{"bids":[[1.0,2.0]],"asks":[[1.1,3.0]],"version":2}}"#
                    )
                )
                .unwrap(),
            MarketEvent::OrderBookDelta(_)
        ));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "public.ws.market.bbo",
                    &Bytes::from_static(br#"{"ch":"x","tick":{"bid":1.0,"ask":1.1}}"#)
                )
                .unwrap(),
            MarketEvent::Bbo { .. }
        ));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "public.ws.market.detail",
                    &Bytes::from_static(br#"{"ch":"x","tick":{"close":1.0}}"#)
                )
                .unwrap(),
            MarketEvent::Ticker { .. }
        ));
        assert!(matches!(adapter.parse_market_event("public.ws.market.trade.detail", &Bytes::from_static(br#"{"tick":{"data":[{"id":"1","price":1.0,"amount":2.0,"direction":"buy"}]}}"#)).unwrap(), MarketEvent::Trade(_)));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "private.ws.accounts.update",
                    &Bytes::from_static(br#"{"ch":"accounts.update","data":[{"currency":"btc"}]}"#)
                )
                .unwrap(),
            MarketEvent::AccountUpdate { .. }
        ));
        assert!(matches!(
            adapter
                .parse_market_event(
                    "private.ws.trade.clearing",
                    &Bytes::from_static(br#"{"ch":"trade.clearing","data":{"status":"done"}}"#)
                )
                .unwrap(),
            MarketEvent::TradeClearing { .. }
        ));
    }

    #[test]
    fn no_secret_leak_in_logs() {
        use tracing::subscriber::with_default;
        use tracing_subscriber::fmt::MakeWriter;

        #[derive(Clone, Default)]
        struct SharedBuf(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for SharedBuf {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl<'a> MakeWriter<'a> for SharedBuf {
            type Writer = SharedBuf;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let out = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(out.clone())
            .finish();
        with_default(subscriber, || info!(key_id = "kid-1", "ws auth"));

        let txt = String::from_utf8(out.0.lock().unwrap().clone()).unwrap();
        assert!(txt.contains("kid-1"));
        assert!(!txt.contains("api_secret"));
        assert!(!txt.contains("secret"));
    }

    #[test]
    fn strict_coverage_gate_is_enabled_for_bittrade() {
        let manifest =
            load_coverage_manifest(std::path::Path::new("../../coverage/bittrade.yaml")).unwrap();
        assert!(manifest.strict);
        assert!(matches!(
            run_coverage_gate(&manifest),
            CoverageGateResult::Passed
        ));
    }
}
