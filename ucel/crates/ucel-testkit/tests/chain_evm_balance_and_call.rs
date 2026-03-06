use ucel_chain_ethereum::balance::{get_erc20_balance, get_native_balance};
use ucel_core::{EvmAddress, EvmBlockRef, EvmChainId};
use ucel_testkit::chain_evm::{provider_set, FakeHttpProvider};

#[test]
fn balance_and_call_parse_and_validate() {
    let p = FakeHttpProvider::new("p", 1);
    p.push_ok(serde_json::json!("0xde0b6b3a7640000"));
    p.push_ok(serde_json::json!("0x64"));
    let f = FakeHttpProvider::new("f", 1);
    let set = provider_set(p, f, 1);

    let addr = EvmAddress("0x1111111111111111111111111111111111111111".into());
    let token = EvmAddress("0x2222222222222222222222222222222222222222".into());
    let native = get_native_balance(&set, EvmChainId(1), addr.clone(), EvmBlockRef::Latest).unwrap();
    assert!(native.wei > 0);
    let erc20 = get_erc20_balance(&set, EvmChainId(1), addr, token, EvmBlockRef::Safe).unwrap();
    assert_eq!(erc20.amount, 100);
}
