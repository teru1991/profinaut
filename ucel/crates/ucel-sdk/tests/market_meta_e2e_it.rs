use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use ucel_core::order_gate::OrderGate;
use ucel_core::Decimal;
use ucel_sdk::market_meta::{MarketMetaService, MarketMetaServiceConfig};
use ucel_sdk::order_normalize::normalize_limit_from_store;
use ucel_symbol_adapter::market_meta::{
    MarketMetaAdapterError, MarketMetaConnectorCapabilities, MarketMetaContext, MarketMetaFetcher,
    MarketMetaRateLimitPolicy,
};
use ucel_symbol_core::{
    Exchange, MarketMeta, MarketMetaId, MarketMetaSnapshot, MarketType, OrderSide,
};

struct MockFetcher {
    snap: MarketMetaSnapshot,
}

#[async_trait]
impl MarketMetaFetcher for MockFetcher {
    fn capabilities(&self) -> MarketMetaConnectorCapabilities {
        MarketMetaConnectorCapabilities {
            supports_rest_snapshot: true,
            supports_incremental_rest: false,
            market_types: vec![MarketType::Spot],
        }
    }

    fn rate_limit_policy(&self) -> MarketMetaRateLimitPolicy {
        MarketMetaRateLimitPolicy {
            max_inflight: 1,
            base_backoff_ms: 10,
            max_backoff_ms: 100,
            jitter: false,
        }
    }

    async fn fetch_market_meta_snapshot(
        &self,
        _ctx: &MarketMetaContext,
    ) -> Result<MarketMetaSnapshot, MarketMetaAdapterError> {
        Ok(self.snap.clone())
    }
}

fn d(s: &str) -> Decimal {
    Decimal::from_str_exact(s).unwrap()
}

#[tokio::test]
async fn e2e_preload_store_normalize_buy_sell() {
    let id = MarketMetaId::new(Exchange::Gmocoin, MarketType::Spot, "BTC");
    let mut meta = MarketMeta::new(id.clone(), d("1"), d("0.0001"));
    meta.min_qty = Some(d("0.0001"));
    meta.min_notional = Some(d("100"));
    meta.validate_meta().unwrap();

    let snap = MarketMetaSnapshot::new_rest(vec![meta]);

    let fetcher: Arc<dyn MarketMetaFetcher> = Arc::new(MockFetcher { snap });
    let cfg = MarketMetaServiceConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(3600),
        require_preload_success: true,
    };
    let svc = Arc::new(MarketMetaService::new(vec![fetcher], cfg));

    svc.preload().await.unwrap();
    let store = svc.store();
    let gate = OrderGate::default();

    let (p, q) = normalize_limit_from_store(
        &store,
        &gate,
        &id,
        OrderSide::Buy,
        d("1000.9"),
        d("0.123456"),
    )
    .unwrap();
    assert_eq!(p.to_string(), "1000");
    assert_eq!(q.to_string(), "0.1234");

    let (p2, q2) = normalize_limit_from_store(
        &store,
        &gate,
        &id,
        OrderSide::Sell,
        d("1000.1"),
        d("0.123456"),
    )
    .unwrap();
    assert_eq!(p2.to_string(), "1001");
    assert_eq!(q2.to_string(), "0.1234");
}

#[tokio::test]
async fn e2e_meta_not_found_is_error() {
    let cfg = MarketMetaServiceConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(3600),
        require_preload_success: false,
    };
    let svc = Arc::new(MarketMetaService::new(vec![], cfg));
    let store = svc.store();
    let gate = OrderGate::default();

    let id = MarketMetaId::new(Exchange::Gmocoin, MarketType::Spot, "BTC");
    let err = normalize_limit_from_store(&store, &gate, &id, OrderSide::Buy, d("1"), d("1"))
        .err()
        .unwrap();
    let msg = err.to_string();
    assert!(msg.contains("not found"));
}
