use ucel_core::{IrAccessPattern, IrDocumentKey};
use ucel_ir::{IrDocumentDetailRequest, IrDocumentListRequest, IrFetchMode};

#[test]
fn ir_fetch_contract_gate() {
    let _list = IrDocumentListRequest {
        source_id: "s".into(),
        market: ucel_core::IrMarket::Jp,
        issuer_key: None,
    };
    let _detail = IrDocumentDetailRequest {
        source_id: "s".into(),
        key: IrDocumentKey {
            source_id: "s".into(),
            source_document_id: "d".into(),
        },
    };

    let modes = [
        IrFetchMode::Api,
        IrFetchMode::Feed,
        IrFetchMode::Html,
        IrFetchMode::Attachment,
    ];
    assert_eq!(modes.len(), 4);

    let patterns = [
        IrAccessPattern::ApiList,
        IrAccessPattern::FeedPoll,
        IrAccessPattern::HtmlDetail,
        IrAccessPattern::ArtifactDownload,
    ];
    assert_eq!(patterns.len(), 4);
}
