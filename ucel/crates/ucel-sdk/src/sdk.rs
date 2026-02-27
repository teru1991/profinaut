use crate::config::SdkConfig;
use crate::error::SdkResult;
use tracing::info;
use ucel_registry::hub::{ExchangeId, Hub, HubConfig};

#[derive(Clone)]
pub struct Sdk {
    cfg: SdkConfig,
    hub: Hub,
}

impl Sdk {
    pub fn config(&self) -> &SdkConfig {
        &self.cfg
    }

    pub fn hub(&self) -> &Hub {
        &self.hub
    }

    pub fn list_operations(&self, ex: ExchangeId) -> SdkResult<Vec<String>> {
        Ok(self.hub.list_operations(ex)?)
    }

    pub fn list_channels(&self, ex: ExchangeId) -> SdkResult<Vec<String>> {
        Ok(self.hub.list_channels(ex)?)
    }
}

/// Builder that enforces safe defaults and centralizes config.
pub struct SdkBuilder {
    cfg: SdkConfig,
}

impl SdkBuilder {
    pub fn new(cfg: SdkConfig) -> Self {
        Self { cfg }
    }

    pub fn with_hub_config(mut self, hub: HubConfig) -> Self {
        self.cfg.hub = hub;
        self
    }

    pub fn build(self) -> SdkResult<Sdk> {
        let hub = Hub::new(self.cfg.hub.clone())?;

        info!(
            repo_root = %self.cfg.repo_root.display(),
            run_id = %self.cfg.run_id,
            "ucel-sdk initialized"
        );

        Ok(Sdk { cfg: self.cfg, hub })
    }
}
