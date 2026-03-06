use crate::errors::EvmReasonCode;
use crate::types::{EvmProviderInfo, ProviderError, ProviderResponse};
use std::time::Duration;
use ucel_core::EvmChainId;

pub trait EvmHttpProvider: Send + Sync {
    fn info(&self) -> &EvmProviderInfo;
    fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<ProviderResponse, ProviderError>;
}

pub trait EvmWsProvider: Send + Sync {
    fn info(&self) -> &EvmProviderInfo;
    fn subscribe(&self, method: &str, params: serde_json::Value) -> Result<String, ProviderError>;
    fn unsubscribe(&self, id: &str) -> Result<(), ProviderError>;
}

#[derive(Debug, Clone)]
pub struct EvmProviderPolicy {
    pub retry_budget: u8,
    pub timeout: Duration,
}

impl Default for EvmProviderPolicy {
    fn default() -> Self {
        Self { retry_budget: 2, timeout: Duration::from_secs(10) }
    }
}

pub struct EvmProviderSet {
    pub primary_http: Box<dyn EvmHttpProvider>,
    pub fallback_http: Vec<Box<dyn EvmHttpProvider>>,
    pub primary_ws: Option<Box<dyn EvmWsProvider>>,
    pub fallback_ws: Vec<Box<dyn EvmWsProvider>>,
    pub policy: EvmProviderPolicy,
}

impl EvmProviderSet {
    pub fn call_with_failover(
        &self,
        method: &str,
        params: serde_json::Value,
        expected_chain_id: EvmChainId,
    ) -> Result<ProviderResponse, ProviderError> {
        let mut providers: Vec<&dyn EvmHttpProvider> = vec![self.primary_http.as_ref()];
        for p in &self.fallback_http { providers.push(p.as_ref()); }

        let mut attempts = 0u8;
        let mut last_err = ProviderError { provider: "none".into(), reason: EvmReasonCode::ProviderTimeout, message: "no provider".into() };
        for provider in providers {
            if provider.info().chain_id != expected_chain_id {
                last_err = ProviderError {
                    provider: provider.info().id.clone(),
                    reason: EvmReasonCode::ProviderChainMismatch,
                    message: "chain id mismatch".into(),
                };
                continue;
            }
            match provider.rpc_call(method, params.clone()) {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last_err = e;
                    attempts += 1;
                    if attempts > self.policy.retry_budget { break; }
                }
            }
        }
        Err(last_err)
    }
}
