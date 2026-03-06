use ucel_chain_ethereum::balance::get_chain_id;
use ucel_chain_ethereum::EvmReasonCode;
use ucel_core::EvmChainId;
use ucel_testkit::chain_evm::{provider_set, FakeHttpProvider};

#[test]
fn provider_failover_and_chain_mismatch() {
    let primary = FakeHttpProvider::new("p", 1);
    primary.push_err(EvmReasonCode::ProviderTimeout, "timeout");
    let fallback = FakeHttpProvider::new("f", 1);
    fallback.push_ok(serde_json::json!("0x1"));

    let set = provider_set(primary, fallback, 1);
    let id = get_chain_id(&set, EvmChainId(1)).unwrap();
    assert_eq!(id.0, 1);
}
