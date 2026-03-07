use ucel_chain_ethereum::{
    build_transaction, estimate_eip1559, send_raw_transaction, sign_transaction, wait_for_receipt,
    DeterministicTestSigner, FeePolicy, FinalityPolicy,
};
use ucel_core::{EvmAddress, EvmChainId};
use ucel_testkit::chain_evm::{provider_set, FakeHttpProvider};

#[test]
fn tx_lifecycle_build_sign_send_receipt() {
    let p = FakeHttpProvider::new("p", 1);
    p.push_ok(serde_json::json!("0xdeadbeef"));
    p.push_ok(serde_json::json!({"blockNumber":"0x10","status":"0x1"}));
    let f = FakeHttpProvider::new("f", 1);
    let set = provider_set(p, f, 1);

    let tx = build_transaction(
        EvmChainId(1),
        EvmAddress("0x1111111111111111111111111111111111111111".into()),
        Some(EvmAddress(
            "0x2222222222222222222222222222222222222222".into(),
        )),
        "0x".into(),
        0,
        21_000,
        estimate_eip1559(10, 2, 21_000, FeePolicy::default()).unwrap(),
        1,
    )
    .unwrap();
    let signer = DeterministicTestSigner {
        signer_id: "s".into(),
    };
    let signed = sign_transaction(&signer, &tx).unwrap();
    let hash = send_raw_transaction(&set, EvmChainId(1), &signed).unwrap();
    let receipt =
        wait_for_receipt(&set, EvmChainId(1), &hash, 32, FinalityPolicy::default(), 1).unwrap();
    assert!(receipt.success);
}
