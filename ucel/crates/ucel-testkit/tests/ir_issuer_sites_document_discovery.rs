use ucel_core::IrMarket;
use ucel_ir::{jp_issuer_feed_adapter, us_issuer_html_adapter, IrDocumentDetailRequest, IrDocumentListRequest, IrSourceAdapter};

#[test]
fn ir_issuer_sites_document_discovery() {
    let jp_feed = jp_issuer_feed_adapter();
    let listed = jp_feed
        .list_documents(&IrDocumentListRequest { source_id: "jp_issuer_ir_feed_public".into(), market: IrMarket::Jp, issuer_key: Some("JP-ACME-1111".into()) })
        .expect("list jp feed docs");
    assert!(!listed.documents.is_empty());

    let us_html = us_issuer_html_adapter();
    let docs = us_html
        .list_documents(&IrDocumentListRequest { source_id: "us_issuer_ir_html_public".into(), market: IrMarket::Us, issuer_key: None })
        .expect("list us docs");
    let detail = us_html
        .fetch_document_detail(&IrDocumentDetailRequest { source_id: "us_issuer_ir_html_public".into(), key: docs.documents[0].key.clone() })
        .expect("detail");
    assert!(detail.source_metadata.get("source_page_url").is_some());
    assert!(detail.source_metadata.get("access_pattern").is_some());
}
