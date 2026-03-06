use ucel_chain_ethereum::fees::{estimate_eip1559, estimate_legacy, FeePolicy};
use ucel_chain_ethereum::NonceManager;

#[test]
fn nonce_reservation_and_fee_modes() {
    let mut nm = NonceManager::default();
    let n1 = nm.reserve(1, "0xabc", 5);
    let n2 = nm.reserve(1, "0xabc", 5);
    assert_eq!(n1, 5);
    assert_eq!(n2, 6);

    let e1559 = estimate_eip1559(10, 2, 21_000, FeePolicy::default()).unwrap();
    assert!(e1559.max_fee_per_gas.is_some());
    let legacy = estimate_legacy(10, 21_000, FeePolicy::default()).unwrap();
    assert!(legacy.legacy_gas_price.is_some());
}
