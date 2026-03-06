use ucel_sdk::equity::EquityDataFacade;

fn main() {
    let facade = EquityDataFacade::default();
    let plan = facade.preview_equity_vendor_plan();
    let quote = facade.get_quote("7203.T").unwrap();
    let bars = facade.get_bars("7203.T", "1d", 1).unwrap();
    println!("plan={} quote_last={} bars={}", plan, quote.last, bars.len());
}
