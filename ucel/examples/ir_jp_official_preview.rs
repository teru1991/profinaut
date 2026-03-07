use ucel_sdk::IrFacade;

fn main() {
    let facade = IrFacade;
    let sources = facade
        .list_ir_sources()
        .expect("list ir sources")
        .into_iter()
        .filter(|s| s.market == "jp")
        .collect::<Vec<_>>();
    println!("jp official candidates={}", sources.len());
    for s in sources {
        println!("{} {} {}", s.source_id, s.source_family, s.access_policy_class);
    }
}
