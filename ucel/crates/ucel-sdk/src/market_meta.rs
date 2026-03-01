use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use ucel_symbol_adapter::market_meta::{MarketMetaContext, MarketMetaFetcher};
use ucel_symbol_store::MarketMetaStore;

#[derive(Debug, Clone)]
pub struct MarketMetaServiceConfig {
    pub ttl: Duration,
    pub refresh_interval: Duration,
    pub require_preload_success: bool,
}

impl Default for MarketMetaServiceConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30 * 60),
            refresh_interval: Duration::from_secs(10 * 60),
            require_preload_success: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum MarketMetaServiceError {
    #[error("preload failed and require_preload_success=true: {0}")]
    PreloadFailed(String),
}

pub struct MarketMetaService {
    store: Arc<MarketMetaStore>,
    fetchers: Vec<Arc<dyn MarketMetaFetcher>>,
    cfg: MarketMetaServiceConfig,
}

impl MarketMetaService {
    pub fn new(fetchers: Vec<Arc<dyn MarketMetaFetcher>>, cfg: MarketMetaServiceConfig) -> Self {
        let store = Arc::new(MarketMetaStore::new(cfg.ttl));
        Self {
            store,
            fetchers,
            cfg,
        }
    }

    pub fn store(&self) -> Arc<MarketMetaStore> {
        Arc::clone(&self.store)
    }

    pub async fn preload(&self) -> Result<(), MarketMetaServiceError> {
        let ctx = MarketMetaContext::default();
        let mut any_success = false;

        for f in &self.fetchers {
            match f.fetch_market_meta_snapshot(&ctx).await {
                Ok(snapshot) => {
                    any_success = true;
                    let events = self.store.apply_snapshot_full(snapshot);
                    info!(events = events.len(), "market_meta preload applied");
                }
                Err(e) => {
                    warn!(error = %e, "market_meta preload failed for a fetcher");
                }
            }
        }

        if self.cfg.require_preload_success && !any_success {
            return Err(MarketMetaServiceError::PreloadFailed(
                "all fetchers failed".to_string(),
            ));
        }
        Ok(())
    }

    pub fn spawn_refresh_loop(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let ctx = MarketMetaContext::default();
            loop {
                tokio::time::sleep(self.cfg.refresh_interval).await;

                for f in &self.fetchers {
                    match f.fetch_market_meta_snapshot(&ctx).await {
                        Ok(snapshot) => {
                            let events = self.store.apply_snapshot_full(snapshot);
                            info!(events = events.len(), "market_meta refresh applied");
                        }
                        Err(e) => {
                            warn!(error = %e, "market_meta refresh failed for a fetcher");
                        }
                    }
                }

                let expired = self.store.gc_expired();
                if !expired.is_empty() {
                    info!(expired = expired.len(), "market_meta gc_expired");
                }
            }
        })
    }
}
