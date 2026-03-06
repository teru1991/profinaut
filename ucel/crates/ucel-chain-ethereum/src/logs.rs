use crate::errors::reason_to_error;
use crate::provider::{EvmProviderSet, EvmWsProvider};
use ucel_core::{EvmAddress, EvmChainId, EvmLogCursor, EvmLogEvent, UcelError};

pub fn get_logs(
    providers: &EvmProviderSet,
    chain_id: EvmChainId,
    address: EvmAddress,
    from_block: u64,
    to_block: u64,
) -> Result<Vec<EvmLogEvent>, UcelError> {
    let filter = serde_json::json!({
        "address": address.0,
        "fromBlock": format!("0x{from_block:x}"),
        "toBlock": format!("0x{to_block:x}"),
    });
    let r = providers
        .call_with_failover("eth_getLogs", serde_json::json!([filter]), chain_id)
        .map_err(|e| reason_to_error(e.reason, e.message))?;
    let arr = r.value.as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();
    for item in arr {
        out.push(EvmLogEvent {
            block_number: u64::from_str_radix(item.get("blockNumber").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16).unwrap_or(0),
            block_hash: item.get("blockHash").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            tx_hash: item.get("transactionHash").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            log_index: u64::from_str_radix(item.get("logIndex").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16).unwrap_or(0),
            address: EvmAddress(item.get("address").and_then(|v| v.as_str()).unwrap_or_default().to_string()),
            topics: item.get("topics").and_then(|v| v.as_array()).cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
            data_hex: item.get("data").and_then(|v| v.as_str()).unwrap_or("0x").to_string(),
            removed: item.get("removed").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    Ok(out)
}

pub fn subscribe_logs(ws: &dyn EvmWsProvider, address: EvmAddress) -> Result<String, UcelError> {
    ws.subscribe("eth_subscribe", serde_json::json!(["logs", {"address": address.0}]))
        .map_err(|e| reason_to_error(e.reason, e.message))
}

pub fn cursor_after(logs: &[EvmLogEvent], current: &EvmLogCursor) -> EvmLogCursor {
    let mut cursor = current.clone();
    if let Some(last) = logs.last() {
        cursor.next_from_block = last.block_number.saturating_add(1);
        cursor.last_block_hash = Some(last.block_hash.clone());
    }
    cursor
}
