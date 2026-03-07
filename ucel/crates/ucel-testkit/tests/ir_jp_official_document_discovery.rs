use ucel_core::IrMarket;
use ucel_ir::{statutory_adapter, timely_adapter, IrDocumentListRequest, IrSourceAdapter};

#[test]
fn ir_jp_official_document_discovery() {
    let s = statutory_adapter();
    let docs = s
        .list_documents(&IrDocumentListRequest {
            source_id: "edinet_api_documents_v2".into(),
            market: IrMarket::Jp,
            issuer_key: None,
        })
        .expect("statutory docs");
    assert!(!docs.documents.is_empty());

    let t = timely_adapter();
    let docs_t = t
        .list_documents(&IrDocumentListRequest {
            source_id: "jp_tdnet_timely_html".into(),
            market: IrMarket::Jp,
            issuer_key: None,
        })
        .expect("timely docs");
    assert!(!docs_t.documents.is_empty());
}
