use ucel_equity_core::vendor::EquityVendorAdapter;
use ucel_testkit::equity::demo_adapter;

#[test]
fn corporate_actions_are_canonicalized() {
    let adapter = demo_adapter();
    let actions = adapter
        .get_corporate_actions("7203.T", "2026-01-01", "2026-12-31")
        .unwrap();
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        ucel_core::EquityCorporateAction::Dividend { dividend, .. } => {
            assert!(dividend.cash_amount > 0.0)
        }
        _ => panic!("unexpected action"),
    }
}
