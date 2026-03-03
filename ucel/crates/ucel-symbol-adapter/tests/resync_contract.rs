use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::watch;
use ucel_symbol_adapter::{
    subscribe_with_optional_resync, ConnectorCapabilities, MappingQuality, RateLimitPolicy,
    ResyncHint, ResyncSignal, SymbolAdapterError, SymbolContext, SymbolEventStream,
    SymbolSubscriber, SymbolSubscriberExtResync,
};
use ucel_symbol_core::{Exchange, InstrumentId, MarketType};
use ucel_symbol_store::SymbolEvent;

#[derive(Clone)]
struct DummySub {
    signal: ResyncSignal,
}

impl DummySub {
    fn new() -> Self {
        Self {
            signal: ResyncSignal::new(),
        }
    }
}

#[async_trait]
impl SymbolSubscriber for DummySub {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            supports_rest_snapshot: true,
            supports_ws_events: true,
            supports_incremental_rest: false,
            market_types: vec![MarketType::Spot],
            symbol_status_mapping_quality: MappingQuality::Exact,
        }
    }

    fn rate_limit_policy(&self) -> RateLimitPolicy {
        RateLimitPolicy {
            max_inflight: 1,
            base_backoff_ms: 10,
            max_backoff_ms: 100,
            jitter: false,
        }
    }

    async fn subscribe_events(
        &self,
        _ctx: &SymbolContext,
    ) -> Result<SymbolEventStream, SymbolAdapterError> {
        let stream = futures_util::stream::iter(vec![dummy_event()]);
        Ok(Box::pin(stream))
    }
}

impl SymbolSubscriberExtResync for DummySub {
    fn resync_receiver(&self) -> Option<watch::Receiver<Option<ResyncHint>>> {
        Some(self.signal.receiver())
    }
}

#[tokio::test]
async fn resync_signal_is_observable() {
    let sub = Arc::new(DummySub::new());
    let ctx = SymbolContext::default();
    let (_stream, rx) = subscribe_with_optional_resync(sub.clone(), &ctx)
        .await
        .expect("subscribe succeeds");
    let mut rx = rx.expect("receiver must exist");

    sub.signal.notify(ResyncHint::Lagged {
        reason: "queue_overflow",
    });
    rx.changed().await.unwrap();
    let v = rx.borrow().clone();
    assert!(matches!(v, Some(ResyncHint::Lagged { .. })));
}

fn dummy_event() -> SymbolEvent {
    SymbolEvent::Removed {
        id: InstrumentId {
            exchange: Exchange::Binance,
            market_type: MarketType::Spot,
            raw_symbol: "DUMMY".into(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        last_known: None,
        reason: Some("dummy".into()),
        ts_recv: std::time::SystemTime::UNIX_EPOCH,
        store_version: 1,
    }
}
