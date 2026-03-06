use ucel_sdk::equity::EquityDataFacade;

fn main() {
    let facade = EquityDataFacade::default();
    let cal = facade.get_market_calendar("JP", "2026-03-06").unwrap();
    let actions = facade
        .get_corporate_actions("7203.T", "2026-01-01", "2026-12-31")
        .unwrap();
    println!("timezone={} sessions={} actions={}", cal.timezone, cal.sessions.len(), actions.len());
}
