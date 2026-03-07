use ucel_core::{IrDocumentFamily, IrDocumentKey, IrMarket};
use ucel_ir::{sec_adapter, IrDocumentDetailRequest, IrDocumentListRequest, IrSourceAdapter};

#[test]
fn ir_us_official_document_discovery() {
    let a = sec_adapter();
    let listed = a
        .list_documents(&IrDocumentListRequest {
            source_id: "sec_edgar_submissions_api".into(),
            market: IrMarket::Us,
            issuer_key: None,
        })
        .expect("list documents");
    assert!(listed.documents.iter().any(|d| d.family == IrDocumentFamily::StatutoryAnnual));
    assert!(listed.documents.iter().any(|d| d.family == IrDocumentFamily::StatutoryQuarterly));
    assert!(listed.documents.iter().any(|d| d.family == IrDocumentFamily::StatutoryCurrent));
    assert!(listed.documents.iter().any(|d| d.family == IrDocumentFamily::Proxy));
    assert!(listed.documents.iter().any(|d| d.family == IrDocumentFamily::MiscIrDocument));

    let detail = a
        .fetch_document_detail(&IrDocumentDetailRequest {
            source_id: "sec_edgar_submissions_api".into(),
            key: IrDocumentKey {
                source_id: "sec_edgar_submissions_api".into(),
                source_document_id: "0000320193-25-000081".into(),
            },
        })
        .expect("detail");
    assert!(detail.source_metadata.get("filing_form").is_some());
    assert!(detail.source_metadata.get("access_pattern").is_some());
}
