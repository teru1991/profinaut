use ucel_sdk::IrFacade;

fn main() {
    let facade = IrFacade;
    let sources = facade.list_ir_sources().expect("list sources");
    let issuer_site_sources = sources
        .into_iter()
        .filter(|s| s.source_family == "jp_issuer_ir_site" || s.source_family == "us_issuer_ir_site")
        .collect::<Vec<_>>();
    println!("issuer-site sources: {}", issuer_site_sources.len());
    for s in issuer_site_sources {
        println!("- {} ({})", s.source_id, s.market);
    }
}
