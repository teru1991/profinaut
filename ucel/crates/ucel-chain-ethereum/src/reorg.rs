use ucel_core::{EvmLogCursor, EvmLogEvent, EvmReorgEvent};

pub fn detect_reorg(cursor: &EvmLogCursor, next: &EvmLogEvent) -> Option<EvmReorgEvent> {
    if next.removed {
        return Some(EvmReorgEvent {
            detected_at_block: next.block_number,
            rollback_to_block: next.block_number.saturating_sub(1),
            depth: 1,
        });
    }
    if let Some(prev_hash) = &cursor.last_block_hash {
        if next.block_number == cursor.next_from_block.saturating_sub(1) && &next.block_hash != prev_hash {
            return Some(EvmReorgEvent {
                detected_at_block: next.block_number,
                rollback_to_block: next.block_number.saturating_sub(1),
                depth: 1,
            });
        }
    }
    None
}

pub fn replay_range_for_reorg(current_from: u64, rollback_to: u64) -> (u64, u64) {
    (rollback_to, current_from)
}
