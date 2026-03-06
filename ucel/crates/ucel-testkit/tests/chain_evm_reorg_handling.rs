use ucel_chain_ethereum::{detect_reorg, replay_range_for_reorg};
use ucel_core::{EvmAddress, EvmLogCursor, EvmLogEvent};

#[test]
fn reorg_detection_and_replay_range() {
    let cursor = EvmLogCursor {
        next_from_block: 11,
        last_safe_block: Some(10),
        last_block_hash: Some("0xaaa".into()),
    };
    let log = EvmLogEvent {
        block_number: 10,
        block_hash: "0xbbb".into(),
        tx_hash: "0x1".into(),
        log_index: 0,
        address: EvmAddress("0x1111111111111111111111111111111111111111".into()),
        topics: vec![],
        data_hex: "0x".into(),
        removed: false,
    };
    let reorg = detect_reorg(&cursor, &log).unwrap();
    let range = replay_range_for_reorg(cursor.next_from_block, reorg.rollback_to_block);
    assert!(range.0 <= range.1);
}
