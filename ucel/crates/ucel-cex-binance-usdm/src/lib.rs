use bytes::Bytes;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, RequestContext, WsConnectRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsChannelSpec {
    pub id: &'static str,
    pub ws_url: &'static str,
    pub channel: &'static str,
    pub requires_auth: bool,
}

pub const WS_CHANNELS: [WsChannelSpec; 10] = [
    WsChannelSpec {
        id: "usdm.public.ws.market.root",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "!markPrice@arr",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.aggtrade",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@aggTrade",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.markprice",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@markPrice",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.kline",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@kline_<interval>",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.bookticker",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@bookTicker",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.liquidation",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@forceOrder",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.depth.partial",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@depth<levels>",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.market.depth.diff",
        ws_url: "wss://fstream.binance.com/ws",
        channel: "<symbol>@depth",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.public.ws.wsapi.general",
        ws_url: "wss://ws-fapi.binance.com/ws-fapi/v1",
        channel: "rpc",
        requires_auth: false,
    },
    WsChannelSpec {
        id: "usdm.private.ws.userdata.events",
        ws_url: "wss://fstream.binance.com/ws/<listenKey>",
        channel: "userdata",
        requires_auth: true,
    },
];

#[derive(Debug, Clone, Default)]
pub struct WsMetrics {
    pub reconnect_total: u64,
    pub resubscribe_total: u64,
    pub ws_drop_total: u64,
    pub ws_orderbook_gap_total: u64,
}

#[derive(Debug)]
pub struct WsSession {
    subscribed: HashSet<String>,
    pub metrics: WsMetrics,
    tx: mpsc::Sender<Bytes>,
    pub dropped_messages: u64,
}

impl WsSession {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<Bytes>) {
        let (tx, rx) = mpsc::channel(capacity);
        (
            Self {
                subscribed: HashSet::new(),
                metrics: WsMetrics::default(),
                tx,
                dropped_messages: 0,
            },
            rx,
        )
    }

    pub fn subscribe(&mut self, channel: &str) {
        self.subscribed.insert(channel.to_string());
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.subscribed.remove(channel);
    }

    pub fn reconnect_and_resubscribe(&mut self) -> Vec<String> {
        self.metrics.reconnect_total += 1;
        self.metrics.resubscribe_total += self.subscribed.len() as u64;
        let mut channels: Vec<_> = self.subscribed.iter().cloned().collect();
        channels.sort();
        channels
    }

    pub fn push_event(&mut self, payload: Bytes) {
        if self.tx.try_send(payload).is_err() {
            self.dropped_messages += 1;
            self.metrics.ws_drop_total += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "e")]
pub enum UsdmWsEvent {
    #[serde(rename = "aggTrade")]
    AggTrade { s: String, p: String, q: String },
    #[serde(rename = "markPriceUpdate")]
    MarkPrice { s: String, p: String },
    #[serde(rename = "kline")]
    Kline { s: String, k: KlinePayload },
    #[serde(rename = "bookTicker")]
    BookTicker { s: String, b: String, a: String },
    #[serde(rename = "forceOrder")]
    Liquidation { o: LiquidationOrder },
    #[serde(rename = "depthUpdate")]
    DepthDiff {
        s: String,
        U: u64,
        u: u64,
        b: Vec<[String; 2]>,
        a: Vec<[String; 2]>,
    },
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    UserOrderUpdate { i: String },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KlinePayload {
    pub i: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LiquidationOrder {
    pub s: String,
}

#[derive(Debug, Clone)]
pub struct OrderBookSync {
    pub last_update_id: u64,
    pub bids: BTreeMap<String, String>,
    pub asks: BTreeMap<String, String>,
    pub degraded: bool,
}

impl Default for OrderBookSync {
    fn default() -> Self {
        Self {
            last_update_id: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            degraded: false,
        }
    }
}

impl OrderBookSync {
    pub fn apply_snapshot(
        &mut self,
        last_update_id: u64,
        bids: Vec<[String; 2]>,
        asks: Vec<[String; 2]>,
    ) {
        self.last_update_id = last_update_id;
        self.bids = bids
            .into_iter()
            .map(|x| (x[0].clone(), x[1].clone()))
            .collect();
        self.asks = asks
            .into_iter()
            .map(|x| (x[0].clone(), x[1].clone()))
            .collect();
        self.degraded = false;
    }

    pub fn apply_diff(
        &mut self,
        first_id: u64,
        final_id: u64,
        bids: Vec<[String; 2]>,
        asks: Vec<[String; 2]>,
        metrics: &mut WsMetrics,
    ) {
        if self.last_update_id == 0
            || first_id > self.last_update_id + 1
            || final_id < self.last_update_id
        {
            self.degraded = true;
            metrics.ws_orderbook_gap_total += 1;
            return;
        }
        self.last_update_id = final_id;
        for level in bids {
            self.bids.insert(level[0].clone(), level[1].clone());
        }
        for level in asks {
            self.asks.insert(level[0].clone(), level[1].clone());
        }
    }

    pub fn resync(&mut self, snapshot_last_update_id: u64) {
        self.last_update_id = snapshot_last_update_id;
        self.degraded = false;
    }
}

pub fn preflight_ws_connect(
    channel_id: &str,
    key_id: Option<String>,
) -> Result<RequestContext, UcelError> {
    let spec = WS_CHANNELS
        .iter()
        .find(|s| s.id == channel_id)
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
    let ctx = RequestContext {
        trace_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        run_id: uuid::Uuid::new_v4().to_string(),
        op: if spec.id.contains("depth") {
            OpName::SubscribeOrderbook
        } else {
            OpName::FetchStatus
        },
        venue: "binance-usdm".into(),
        policy_id: "default".into(),
        key_id: if spec.requires_auth { key_id } else { None },
        requires_auth: spec.requires_auth,
    };
    enforce_auth_boundary(&ctx)?;
    Ok(ctx)
}

pub fn build_connect_request(channel_id: &str) -> Result<WsConnectRequest, UcelError> {
    let spec = WS_CHANNELS
        .iter()
        .find(|s| s.id == channel_id)
        .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
    info!(channel = %spec.id, "ws connect");
    Ok(WsConnectRequest {
        url: spec.ws_url.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};

    #[test]
    fn ws_contract_covers_all_catalog_ids() {
        let ids: HashSet<&str> = WS_CHANNELS.iter().map(|s| s.id).collect();
        for expected in [
            "usdm.public.ws.market.root",
            "usdm.public.ws.market.aggtrade",
            "usdm.public.ws.market.markprice",
            "usdm.public.ws.market.kline",
            "usdm.public.ws.market.bookticker",
            "usdm.public.ws.market.liquidation",
            "usdm.public.ws.market.depth.partial",
            "usdm.public.ws.market.depth.diff",
            "usdm.public.ws.wsapi.general",
            "usdm.private.ws.userdata.events",
        ] {
            assert!(ids.contains(expected));
        }
    }

    #[test]
    fn private_preflight_rejects_without_auth() {
        let err = preflight_ws_connect("usdm.private.ws.userdata.events", None).unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
    }

    #[test]
    fn reconnect_and_resubscribe_is_idempotent() {
        let (mut session, _rx) = WsSession::new(2);
        session.subscribe("a");
        session.subscribe("a");
        session.subscribe("b");
        let set = session.reconnect_and_resubscribe();
        assert_eq!(set, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(session.metrics.resubscribe_total, 2);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bounded_backpressure_drops_on_overflow() {
        let (mut session, mut rx) = WsSession::new(1);
        session.push_event(Bytes::from_static(b"1"));
        session.push_event(Bytes::from_static(b"2"));
        assert_eq!(session.dropped_messages, 1);
        assert_eq!(rx.recv().await.unwrap(), Bytes::from_static(b"1"));
    }

    #[test]
    fn orderbook_gap_degraded_resync_recovered() {
        let mut ob = OrderBookSync::default();
        let mut m = WsMetrics::default();
        ob.apply_snapshot(100, vec![["1".into(), "1".into()]], vec![]);
        ob.apply_diff(105, 106, vec![], vec![], &mut m);
        assert!(ob.degraded);
        assert_eq!(m.ws_orderbook_gap_total, 1);
        ob.resync(200);
        assert!(!ob.degraded);
    }

    #[test]
    fn typed_deserialize_no_value() {
        let event: UsdmWsEvent =
            serde_json::from_str(r#"{"e":"aggTrade","s":"BTCUSDT","p":"100","q":"1"}"#).unwrap();
        match event {
            UsdmWsEvent::AggTrade { s, .. } => assert_eq!(s, "BTCUSDT"),
            _ => panic!(),
        }
    }

    #[test]
    fn strict_coverage_gate_for_binance_usdm_has_no_gaps() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../coverage/binance-usdm.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(manifest.strict);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }
}
