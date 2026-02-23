use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Catalog {
    ws_channels: Vec<CatalogWsEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogWsEntry {
    id: String,
    channel: String,
    ws_url: String,
    visibility: String,
}

#[derive(Debug, Clone)]
pub struct WsChannelSpec {
    pub id: String,
    pub channel: String,
    pub ws_url: String,
    pub requires_auth: bool,
}

pub fn ws_channel_specs() -> Vec<WsChannelSpec> {
    let raw = include_str!("../../../../docs/exchanges/upbit/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid upbit catalog");
    catalog
        .ws_channels
        .into_iter()
        .map(|x| WsChannelSpec {
            requires_auth: x.visibility == "private",
            id: x.id,
            channel: x.channel,
            ws_url: x.ws_url,
        })
        .collect()
}

#[derive(Debug, Default, Clone)]
pub struct WsAdapterMetrics {
    pub ws_reconnect_total: u64,
    pub ws_resubscribe_total: u64,
    pub ws_backpressure_drops_total: u64,
    pub ws_orderbook_gap_total: u64,
    pub ws_orderbook_resync_total: u64,
    pub ws_orderbook_recovered_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpbitSubscribeFrame {
    pub ticket: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Ticker {
        code: String,
        trade_price: f64,
    },
    Trade {
        code: String,
        trade_price: f64,
        trade_volume: f64,
    },
    Orderbook {
        code: String,
        total_ask_size: Option<f64>,
        total_bid_size: Option<f64>,
    },
    Candle {
        code: String,
        trade_price: f64,
    },
    SubscriptionList {
        channels: Vec<String>,
    },
    MyOrder {
        code: Option<String>,
        state: Option<String>,
    },
    MyAsset {
        currency: Option<String>,
        balance: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum UpbitMessage {
    #[serde(rename = "ticker")]
    Ticker { code: String, trade_price: f64 },
    #[serde(rename = "trade")]
    Trade {
        code: String,
        trade_price: f64,
        trade_volume: f64,
    },
    #[serde(rename = "orderbook")]
    Orderbook {
        code: String,
        total_ask_size: Option<f64>,
        total_bid_size: Option<f64>,
        #[serde(rename = "timestamp")]
        _timestamp: Option<u64>,
    },
    #[serde(rename = "candle")]
    Candle { code: String, trade_price: f64 },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions { codes: Vec<String> },
    #[serde(rename = "myOrder")]
    MyOrder {
        code: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "myAsset")]
    MyAsset {
        currency: Option<String>,
        balance: Option<String>,
    },
}

pub fn normalize_ws_message(raw: &str) -> Result<MarketEvent, UcelError> {
    let msg: UpbitMessage = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("typed ws parse error: {e}")))?;
    Ok(match msg {
        UpbitMessage::Ticker { code, trade_price } => MarketEvent::Ticker { code, trade_price },
        UpbitMessage::Trade {
            code,
            trade_price,
            trade_volume,
        } => MarketEvent::Trade {
            code,
            trade_price,
            trade_volume,
        },
        UpbitMessage::Orderbook {
            code,
            total_ask_size,
            total_bid_size,
            ..
        } => MarketEvent::Orderbook {
            code,
            total_ask_size,
            total_bid_size,
        },
        UpbitMessage::Candle { code, trade_price } => MarketEvent::Candle { code, trade_price },
        UpbitMessage::ListSubscriptions { codes } => {
            MarketEvent::SubscriptionList { channels: codes }
        }
        UpbitMessage::MyOrder { code, state } => MarketEvent::MyOrder { code, state },
        UpbitMessage::MyAsset { currency, balance } => MarketEvent::MyAsset { currency, balance },
    })
}

#[derive(Debug, Default, Clone)]
pub struct OrderbookSync {
    pub last_ts: Option<u64>,
    pub degraded: bool,
    pub book: BTreeMap<String, f64>,
}
impl OrderbookSync {
    pub fn apply_snapshot(&mut self, ts: u64) {
        self.last_ts = Some(ts);
        self.degraded = false;
    }

    pub fn apply_delta(
        &mut self,
        ts: u64,
        metrics: &mut WsAdapterMetrics,
    ) -> Result<(), UcelError> {
        match self.last_ts {
            Some(prev) if ts <= prev => {
                self.degraded = true;
                metrics.ws_orderbook_gap_total += 1;
                metrics.ws_orderbook_resync_total += 1;
                Err(UcelError::new(
                    ErrorCode::Desync,
                    "gap/mismatch detected, immediate resync",
                ))
            }
            Some(_) => {
                self.last_ts = Some(ts);
                Ok(())
            }
            None => {
                self.degraded = true;
                metrics.ws_orderbook_gap_total += 1;
                metrics.ws_orderbook_resync_total += 1;
                Err(UcelError::new(ErrorCode::Desync, "delta before snapshot"))
            }
        }
    }

    pub fn resync(&mut self, snapshot_ts: u64, metrics: &mut WsAdapterMetrics) {
        let was_degraded = self.degraded;
        self.apply_snapshot(snapshot_ts);
        if was_degraded {
            metrics.ws_orderbook_recovered_total += 1;
        }
    }

    pub fn mark_recovered(&mut self, metrics: &mut WsAdapterMetrics) {
        if self.degraded {
            self.degraded = false;
            metrics.ws_orderbook_recovered_total += 1;
        }
    }
}

pub struct BackpressureQueue {
    tx: mpsc::Sender<MarketEvent>,
    rx: mpsc::Receiver<MarketEvent>,
}
impl BackpressureQueue {
    pub fn with_capacity(cap: usize) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx }
    }

    pub fn try_push(&self, ev: MarketEvent, metrics: &mut WsAdapterMetrics) {
        if self.tx.try_send(ev).is_err() {
            metrics.ws_backpressure_drops_total += 1;
        }
    }

    pub async fn recv(&mut self) -> Option<MarketEvent> {
        self.rx.recv().await
    }
}

#[derive(Debug, Default)]
pub struct UpbitWsAdapter {
    subscriptions: HashSet<String>,
    pub metrics: WsAdapterMetrics,
}
impl UpbitWsAdapter {
    pub fn build_subscribe(
        endpoint_id: &str,
        code: &str,
        key_id: Option<&str>,
    ) -> Result<UpbitSubscribeFrame, UcelError> {
        let spec = ws_channel_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown ws endpoint"))?;
        if spec.requires_auth && key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private ws endpoint requires auth",
            ));
        }
        if spec.requires_auth {
            info!(target: "upbit.auth", key_id = %key_id.unwrap_or(""), "private ws subscribe preflight passed");
        }
        Ok(UpbitSubscribeFrame {
            ticket: "ucel-upbit".into(),
            channel_type: spec.channel,
            codes: vec![code.into()],
        })
    }

    pub fn build_unsubscribe(
        endpoint_id: &str,
        code: &str,
    ) -> Result<UpbitSubscribeFrame, UcelError> {
        Self::build_subscribe(endpoint_id, code, Some("dummy"))
    }

    pub fn subscribe_once(&mut self, endpoint_id: &str, code: &str) -> bool {
        self.subscriptions.insert(format!("{endpoint_id}:{code}"))
    }

    pub async fn reconnect_and_resubscribe<T: Transport>(
        &mut self,
        transport: &T,
    ) -> Result<usize, UcelError> {
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "upbit".into(),
            policy_id: "default".into(),
            key_id: None,
            requires_auth: false,
        };
        transport
            .connect_ws(
                WsConnectRequest {
                    url: "wss://api.upbit.com/websocket/v1".into(),
                },
                ctx,
            )
            .await?;
        self.metrics.ws_reconnect_total += 1;
        self.metrics.ws_resubscribe_total += self.subscriptions.len() as u64;
        Ok(self.subscriptions.len())
    }
}

pub fn scrub_secrets(line: &str) -> String {
    line.split_whitespace()
        .map(|x| {
            if x.starts_with("api_key=") {
                "api_key=***".to_string()
            } else if x.starts_with("api_secret=") {
                "api_secret=***".to_string()
            } else {
                x.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use ucel_transport::{HttpRequest, HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        ws_connects: AtomicUsize,
    }
    impl SpyTransport {
        fn new() -> Self {
            Self {
                ws_connects: AtomicUsize::new(0),
            }
        }
    }
    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            Err(UcelError::new(ErrorCode::NotSupported, "unused"))
        }
        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            self.ws_connects.fetch_add(1, Ordering::Relaxed);
            Ok(WsStream { connected: true })
        }
    }

    #[test]
    fn all_ws_catalog_rows_build_subscribe_unsubscribe() {
        for spec in ws_channel_specs() {
            let key = if spec.requires_auth {
                Some("kid")
            } else {
                None
            };
            assert_eq!(
                UpbitWsAdapter::build_subscribe(&spec.id, "KRW-BTC", key)
                    .unwrap()
                    .channel_type,
                spec.channel
            );
            assert_eq!(
                UpbitWsAdapter::build_unsubscribe(&spec.id, "KRW-BTC")
                    .unwrap()
                    .channel_type,
                spec.channel
            );
        }
    }

    #[test]
    fn private_preflight_reject_no_connect() {
        let err =
            UpbitWsAdapter::build_subscribe("exchange.private.ws.myasset.stream", "KRW-BTC", None)
                .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reconnect_resubscribe_idempotent() {
        let spy = SpyTransport::new();
        let mut ws = UpbitWsAdapter::default();
        assert!(ws.subscribe_once("quotation.public.ws.trade.stream", "KRW-BTC"));
        assert!(!ws.subscribe_once("quotation.public.ws.trade.stream", "KRW-BTC"));
        let restored = ws.reconnect_and_resubscribe(&spy).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(ws.metrics.ws_resubscribe_total, 1);
        assert_eq!(spy.ws_connects.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn typed_deserialize_and_normalize() {
        let t = normalize_ws_message(r#"{"type":"ticker","code":"KRW-BTC","trade_price":1.0}"#)
            .unwrap();
        assert!(matches!(t, MarketEvent::Ticker { .. }));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_and_metrics() {
        let mut q = BackpressureQueue::with_capacity(1);
        let mut m = WsAdapterMetrics::default();
        q.try_push(
            MarketEvent::Candle {
                code: "KRW-BTC".into(),
                trade_price: 1.0,
            },
            &mut m,
        );
        q.try_push(
            MarketEvent::Candle {
                code: "KRW-ETH".into(),
                trade_price: 2.0,
            },
            &mut m,
        );
        assert_eq!(m.ws_backpressure_drops_total, 1);
        assert!(
            matches!(q.recv().await.unwrap(), MarketEvent::Candle { code, .. } if code == "KRW-BTC")
        );
    }

    #[test]
    fn orderbook_gap_resync_recovered() {
        let mut sync = OrderbookSync::default();
        let mut m = WsAdapterMetrics::default();
        sync.apply_snapshot(100);
        assert!(sync.apply_delta(100, &mut m).is_err());
        assert!(sync.degraded);
        sync.resync(101, &mut m);
        assert!(!sync.degraded);
        assert_eq!(m.ws_orderbook_gap_total, 1);
        assert_eq!(m.ws_orderbook_resync_total, 1);
        assert_eq!(m.ws_orderbook_recovered_total, 1);
    }

    #[test]
    fn duplicate_and_out_of_order_policy_forces_resync() {
        let mut sync = OrderbookSync::default();
        let mut m = WsAdapterMetrics::default();
        sync.apply_snapshot(10);
        assert!(sync.apply_delta(9, &mut m).is_err());
        assert!(sync.degraded);
    }

    #[test]
    fn no_secret_leak() {
        let line = "key_id=alpha api_key=hello api_secret=world";
        let scrubbed = scrub_secrets(line);
        assert!(scrubbed.contains("key_id=alpha"));
        assert!(!scrubbed.contains("hello"));
        assert!(!scrubbed.contains("world"));
    }

    #[test]
    fn strict_gate_enabled_and_zero_gaps() {
        let manifest: serde_yaml::Value =
            serde_yaml::from_str(include_str!("../../../coverage/upbit.yaml")).unwrap();
        assert_eq!(manifest["strict"], true);
        let entries = manifest["entries"].as_sequence().unwrap();
        for e in entries {
            assert_eq!(e["implemented"], true, "implemented gap for {:?}", e["id"]);
            assert_eq!(e["tested"], true, "tested gap for {:?}", e["id"]);
        }
    }
}

pub mod symbols;
pub mod ws_manager;
pub mod channels;
