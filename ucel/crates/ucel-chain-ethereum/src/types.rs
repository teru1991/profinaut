use crate::errors::EvmReasonCode;
use ucel_core::EvmChainId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderResponse {
    pub provider: String,
    pub method: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError {
    pub provider: String,
    pub reason: EvmReasonCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmProviderInfo {
    pub id: String,
    pub chain_id: EvmChainId,
    pub priority: u8,
}
