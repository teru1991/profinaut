use ucel_chain_ethereum::{replay_range_for_reorg, resume_cursor};

fn main() {
    let cursor = resume_cursor(100);
    let range = replay_range_for_reorg(cursor.next_from_block, 95);
    println!("resume_from={} replay={:?}", cursor.next_from_block, range);
}
