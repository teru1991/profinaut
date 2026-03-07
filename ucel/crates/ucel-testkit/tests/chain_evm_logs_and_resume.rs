use ucel_chain_ethereum::{cursor_after, dedup_logs, get_logs, resume_cursor};
use ucel_core::{EvmAddress, EvmChainId};
use ucel_testkit::chain_evm::{provider_set, FakeHttpProvider};

#[test]
fn logs_and_resume_cursor_dedup() {
    let p = FakeHttpProvider::new("p", 1);
    p.push_ok(serde_json::json!([
      {"blockNumber":"0x10","blockHash":"0xaaa","transactionHash":"0x1","logIndex":"0x0","address":"0x1111111111111111111111111111111111111111","topics":[],"data":"0x","removed":false},
      {"blockNumber":"0x10","blockHash":"0xaaa","transactionHash":"0x1","logIndex":"0x0","address":"0x1111111111111111111111111111111111111111","topics":[],"data":"0x","removed":false}
    ]));
    let f = FakeHttpProvider::new("f", 1);
    let set = provider_set(p, f, 1);
    let logs = get_logs(
        &set,
        EvmChainId(1),
        EvmAddress("0x1111111111111111111111111111111111111111".into()),
        1,
        16,
    )
    .unwrap();
    let dedup = dedup_logs(logs);
    assert_eq!(dedup.len(), 1);
    let c = resume_cursor(16);
    let n = cursor_after(&dedup, &c);
    assert_eq!(n.next_from_block, 17);
}
