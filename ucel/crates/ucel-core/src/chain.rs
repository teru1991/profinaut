use crate::{ErrorCode, UcelError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainKind {
    Evm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainSurface {
    ReadOnly,
    TransactionSend,
    Logs,
    Subscriptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvmChainId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvmAddress(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvmBlockRef {
    Latest,
    Pending,
    Safe,
    Finalized,
    Number(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvmFinalityState {
    Pending,
    Unsafe,
    Safe,
    Finalized,
    Reorged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmNativeBalance {
    pub chain_id: EvmChainId,
    pub address: EvmAddress,
    pub wei: u128,
    pub block_ref: EvmBlockRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmTokenBalance {
    pub chain_id: EvmChainId,
    pub address: EvmAddress,
    pub token: EvmAddress,
    pub amount: u128,
    pub block_ref: EvmBlockRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmFeeEstimate {
    pub legacy_gas_price: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmTransactionRequest {
    pub chain_id: EvmChainId,
    pub from: EvmAddress,
    pub to: Option<EvmAddress>,
    pub data_hex: String,
    pub value_wei: u128,
    pub gas_limit: u64,
    pub fee: EvmFeeEstimate,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmSignedTransaction {
    pub tx_hash: String,
    pub raw_tx_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmTransactionReceipt {
    pub tx_hash: String,
    pub block_number: Option<u64>,
    pub success: bool,
    pub confirmations: u64,
    pub finality: EvmFinalityState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmLogEvent {
    pub block_number: u64,
    pub block_hash: String,
    pub tx_hash: String,
    pub log_index: u64,
    pub address: EvmAddress,
    pub topics: Vec<String>,
    pub data_hex: String,
    pub removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmLogCursor {
    pub next_from_block: u64,
    pub last_safe_block: Option<u64>,
    pub last_block_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmReorgEvent {
    pub detected_at_block: u64,
    pub rollback_to_block: u64,
    pub depth: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainSupport {
    Supported,
    Partial,
    NotSupported,
}

pub fn validate_evm_address(addr: &EvmAddress) -> Result<(), UcelError> {
    let s = addr.0.strip_prefix("0x").unwrap_or(&addr.0);
    if s.len() != 40 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "invalid evm address",
        ));
    }
    Ok(())
}

pub fn validate_chain_id(chain_id: EvmChainId) -> Result<(), UcelError> {
    if chain_id.0 == 0 {
        return Err(UcelError::new(ErrorCode::CatalogInvalid, "invalid chain id 0"));
    }
    Ok(())
}

pub fn finality_at_least(current: EvmFinalityState, required: EvmFinalityState) -> bool {
    current >= required && current != EvmFinalityState::Reorged
}

pub fn receipt_is_success(receipt: &EvmTransactionReceipt) -> bool {
    receipt.success && receipt.finality != EvmFinalityState::Reorged
}
