use crate::errors::{reason_to_error, EvmReasonCode};
use crate::provider::EvmProviderSet;
use ucel_core::{
    validate_evm_address, EvmAddress, EvmBlockRef, EvmChainId, EvmNativeBalance, EvmTokenBalance,
    UcelError,
};

fn parse_hex_u128(v: &str) -> Result<u128, UcelError> {
    let clean = v.trim_start_matches("0x");
    u128::from_str_radix(clean, 16)
        .map_err(|_| reason_to_error(EvmReasonCode::ExecutionReverted, "malformed hex quantity"))
}

pub fn get_chain_id(set: &EvmProviderSet, expected: EvmChainId) -> Result<EvmChainId, UcelError> {
    let r = set
        .call_with_failover("eth_chainId", serde_json::json!([]), expected)
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    let s = r.value.as_str().unwrap_or("0x0");
    let id = parse_hex_u128(s)? as u64;
    Ok(EvmChainId(id))
}

pub fn get_block_number(set: &EvmProviderSet, expected: EvmChainId) -> Result<u64, UcelError> {
    let r = set
        .call_with_failover("eth_blockNumber", serde_json::json!([]), expected)
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    Ok(parse_hex_u128(r.value.as_str().unwrap_or("0x0"))? as u64)
}

pub fn get_native_balance(
    set: &EvmProviderSet,
    expected: EvmChainId,
    address: EvmAddress,
    block_ref: EvmBlockRef,
) -> Result<EvmNativeBalance, UcelError> {
    validate_evm_address(&address)?;
    let block = match block_ref {
        EvmBlockRef::Latest => "latest".to_string(),
        EvmBlockRef::Pending => "pending".to_string(),
        EvmBlockRef::Safe => "safe".to_string(),
        EvmBlockRef::Finalized => "finalized".to_string(),
        EvmBlockRef::Number(n) => format!("0x{n:x}"),
    };
    let r = set
        .call_with_failover(
            "eth_getBalance",
            serde_json::json!([address.0, block]),
            expected,
        )
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    let wei = parse_hex_u128(r.value.as_str().unwrap_or("0x0"))?;
    Ok(EvmNativeBalance {
        chain_id: expected,
        address,
        wei,
        block_ref,
    })
}

pub fn get_erc20_balance(
    set: &EvmProviderSet,
    expected: EvmChainId,
    holder: EvmAddress,
    token: EvmAddress,
    block_ref: EvmBlockRef,
) -> Result<EvmTokenBalance, UcelError> {
    validate_evm_address(&holder)?;
    validate_evm_address(&token)?;
    let block = match block_ref {
        EvmBlockRef::Latest => "latest".to_string(),
        EvmBlockRef::Pending => "pending".to_string(),
        EvmBlockRef::Safe => "safe".to_string(),
        EvmBlockRef::Finalized => "finalized".to_string(),
        EvmBlockRef::Number(n) => format!("0x{n:x}"),
    };
    let data = format!("0x70a08231{:0>64}", holder.0.trim_start_matches("0x"));
    let call = serde_json::json!({"to": token.0, "data": data});
    let r = set
        .call_with_failover("eth_call", serde_json::json!([call, block]), expected)
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    let amount = parse_hex_u128(r.value.as_str().unwrap_or("0x0"))?;
    Ok(EvmTokenBalance {
        chain_id: expected,
        address: holder,
        token,
        amount,
        block_ref,
    })
}
