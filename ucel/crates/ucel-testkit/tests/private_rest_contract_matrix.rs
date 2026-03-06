use std::path::Path;

use ucel_core::{PrivateRestOperation, PrivateRestSupport};
use ucel_registry::hub::ExchangeId;
use ucel_testkit::private_rest::{has_required_fixtures, support_for};

#[test]
fn private_rest_contract_matrix_is_consistent() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");

    let supported = [
        ExchangeId::Bitbank,
        ExchangeId::Bitflyer,
        ExchangeId::Coincheck,
        ExchangeId::Gmocoin,
    ];

    for exchange in supported {
        let venue = exchange.as_str();
        assert!(has_required_fixtures(&repo_root, venue));
        let balances = support_for(exchange, PrivateRestOperation::GetBalances, &repo_root);
        let open_orders = support_for(exchange, PrivateRestOperation::GetOpenOrders, &repo_root);
        let cancel = support_for(exchange, PrivateRestOperation::CancelOrder, &repo_root);
        assert!(!matches!(balances, PrivateRestSupport::BlockedByPolicy));
        assert!(!matches!(open_orders, PrivateRestSupport::BlockedByPolicy));
        assert!(!matches!(cancel, PrivateRestSupport::BlockedByPolicy));
    }

    for blocked in [ExchangeId::Sbivc, ExchangeId::Upbit, ExchangeId::Bittrade] {
        let support = support_for(blocked, PrivateRestOperation::GetBalances, &repo_root);
        assert_eq!(support, PrivateRestSupport::BlockedByPolicy);
    }
}
