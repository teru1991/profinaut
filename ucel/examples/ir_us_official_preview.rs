use ucel_sdk::IrFacade;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facade = IrFacade;
    let src = "sec_edgar_submissions_api";

    let issuers = facade.discover_us_official_issuers(src, Some("AAPL".into()))?;
    let (doc_count, _docs, artifacts) = facade.preview_us_official_document_summary(src)?;

    println!("source={src} issuers={} documents={doc_count} artifacts={}", issuers.len(), artifacts.len());
    Ok(())
}
