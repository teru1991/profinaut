use std::collections::HashSet;
use ucel_core::{EvmLogCursor, EvmLogEvent};

pub fn resume_cursor(last_safe_block: u64) -> EvmLogCursor {
    EvmLogCursor {
        next_from_block: last_safe_block,
        last_safe_block: Some(last_safe_block),
        last_block_hash: None,
    }
}

pub fn dedup_logs(logs: Vec<EvmLogEvent>) -> Vec<EvmLogEvent> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for log in logs {
        let key = format!("{}:{}", log.tx_hash, log.log_index);
        if seen.insert(key) {
            out.push(log);
        }
    }
    out
}
