use crate::errors::{reason_to_error, EvmReasonCode};
use crate::finality::{finality_from_confirmations, FinalityPolicy};
use crate::provider::EvmProviderSet;
use ucel_core::{EvmChainId, EvmTransactionReceipt, UcelError};

pub fn get_receipt(
    providers: &EvmProviderSet,
    chain_id: EvmChainId,
    tx_hash: &str,
    latest_block: u64,
    finality: FinalityPolicy,
) -> Result<Option<EvmTransactionReceipt>, UcelError> {
    let r = providers
        .call_with_failover(
            "eth_getTransactionReceipt",
            serde_json::json!([tx_hash]),
            chain_id,
        )
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    if r.value.is_null() {
        return Ok(None);
    }
    let block_number_hex = r
        .value
        .get("blockNumber")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0");
    let bn = u64::from_str_radix(block_number_hex.trim_start_matches("0x"), 16).unwrap_or(0);
    let status = r
        .value
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("0x1");
    let success = status != "0x0";
    let confirmations = latest_block.saturating_sub(bn);
    Ok(Some(EvmTransactionReceipt {
        tx_hash: tx_hash.to_string(),
        block_number: Some(bn),
        success,
        confirmations,
        finality: finality_from_confirmations(confirmations, finality),
    }))
}

pub fn wait_for_receipt(
    providers: &EvmProviderSet,
    chain_id: EvmChainId,
    tx_hash: &str,
    latest_block: u64,
    finality: FinalityPolicy,
    max_polls: u32,
) -> Result<EvmTransactionReceipt, UcelError> {
    for _ in 0..max_polls {
        if let Some(r) = get_receipt(providers, chain_id, tx_hash, latest_block, finality)? {
            return Ok(r);
        }
    }
    Err(reason_to_error(
        EvmReasonCode::ReceiptTimeout,
        "receipt timeout",
    ))
}
