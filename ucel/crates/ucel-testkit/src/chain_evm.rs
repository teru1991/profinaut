use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ucel_chain_ethereum::provider::{EvmHttpProvider, EvmWsProvider};
use ucel_chain_ethereum::{
    EvmProviderInfo, EvmProviderSet, EvmReasonCode, ProviderError, ProviderResponse,
};
use ucel_core::EvmChainId;

#[derive(Clone)]
pub struct FakeHttpProvider {
    pub info: EvmProviderInfo,
    pub responses: Arc<Mutex<VecDeque<Result<serde_json::Value, ProviderError>>>>,
}

impl FakeHttpProvider {
    pub fn new(id: &str, chain_id: u64) -> Self {
        Self {
            info: EvmProviderInfo { id: id.to_string(), chain_id: EvmChainId(chain_id), priority: 0 },
            responses: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push_ok(&self, v: serde_json::Value) {
        self.responses.lock().unwrap().push_back(Ok(v));
    }

    pub fn push_err(&self, reason: EvmReasonCode, msg: &str) {
        self.responses.lock().unwrap().push_back(Err(ProviderError { provider: self.info.id.clone(), reason, message: msg.to_string() }));
    }
}

impl EvmHttpProvider for FakeHttpProvider {
    fn info(&self) -> &EvmProviderInfo { &self.info }

    fn rpc_call(&self, method: &str, _params: serde_json::Value) -> Result<ProviderResponse, ProviderError> {
        let next = self.responses.lock().unwrap().pop_front().unwrap_or_else(|| Err(ProviderError {
            provider: self.info.id.clone(),
            reason: EvmReasonCode::ProviderTimeout,
            message: "no response".into(),
        }))?;
        Ok(ProviderResponse { provider: self.info.id.clone(), method: method.to_string(), value: next })
    }
}

pub struct FakeWsProvider {
    pub info: EvmProviderInfo,
}

impl EvmWsProvider for FakeWsProvider {
    fn info(&self) -> &EvmProviderInfo { &self.info }

    fn subscribe(&self, _method: &str, _params: serde_json::Value) -> Result<String, ProviderError> {
        Ok("sub-1".into())
    }

    fn unsubscribe(&self, _id: &str) -> Result<(), ProviderError> { Ok(()) }
}

pub fn provider_set(primary: FakeHttpProvider, fallback: FakeHttpProvider, chain_id: u64) -> EvmProviderSet {
    EvmProviderSet {
        primary_http: Box::new(primary),
        fallback_http: vec![Box::new(fallback)],
        primary_ws: Some(Box::new(FakeWsProvider { info: EvmProviderInfo { id: "ws".into(), chain_id: EvmChainId(chain_id), priority: 0 } })),
        fallback_ws: vec![],
        policy: Default::default(),
    }
}
