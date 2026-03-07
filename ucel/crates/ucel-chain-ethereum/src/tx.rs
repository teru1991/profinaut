use crate::errors::{reason_to_error, EvmReasonCode};
use crate::provider::EvmProviderSet;
use crate::signer::EvmSigner;
use ucel_core::{
    validate_chain_id, validate_evm_address, EvmAddress, EvmChainId, EvmFeeEstimate,
    EvmSignedTransaction, EvmTransactionRequest, UcelError,
};

#[allow(clippy::too_many_arguments)]
pub fn build_transaction(
    chain_id: EvmChainId,
    from: EvmAddress,
    to: Option<EvmAddress>,
    data_hex: String,
    value_wei: u128,
    gas_limit: u64,
    fee: EvmFeeEstimate,
    nonce: u64,
) -> Result<EvmTransactionRequest, UcelError> {
    validate_chain_id(chain_id)?;
    validate_evm_address(&from)?;
    if let Some(to) = &to {
        validate_evm_address(to)?;
    }
    Ok(EvmTransactionRequest {
        chain_id,
        from,
        to,
        data_hex,
        value_wei,
        gas_limit,
        fee,
        nonce,
    })
}

pub fn sign_transaction(
    signer: &dyn EvmSigner,
    tx: &EvmTransactionRequest,
) -> Result<EvmSignedTransaction, UcelError> {
    signer.sign_transaction(tx)
}

pub fn send_raw_transaction(
    providers: &EvmProviderSet,
    chain_id: EvmChainId,
    signed: &EvmSignedTransaction,
) -> Result<String, UcelError> {
    let r = providers
        .call_with_failover(
            "eth_sendRawTransaction",
            serde_json::json!([signed.raw_tx_hex]),
            chain_id,
        )
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    let tx_hash = r.value.as_str().unwrap_or_default().to_string();
    if !tx_hash.starts_with("0x") {
        return Err(reason_to_error(
            EvmReasonCode::ExecutionReverted,
            "invalid tx hash",
        ));
    }
    Ok(tx_hash)
}
