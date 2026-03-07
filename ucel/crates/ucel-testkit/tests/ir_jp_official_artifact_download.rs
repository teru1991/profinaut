use ucel_core::IrDocumentKey;
use ucel_ir::{
    statutory_adapter, timely_adapter, IrArtifactFetchRequest, IrArtifactListRequest,
    IrSourceAdapter,
};

#[test]
fn ir_jp_official_artifact_download() {
    let s = statutory_adapter();
    let list = s
        .list_artifacts(&IrArtifactListRequest {
            source_id: "edinet_api_documents_v2".into(),
            document_key: IrDocumentKey {
                source_id: "edinet_api_documents_v2".into(),
                source_document_id: "S100AAA1".into(),
            },
        })
        .expect("list artifacts");
    assert!(!list.artifacts.is_empty());

    let fetched = s
        .fetch_artifact(&IrArtifactFetchRequest {
            source_id: "edinet_api_documents_v2".into(),
            artifact_key: list.artifacts[0].key.clone(),
        })
        .expect("fetch artifact");
    assert!(fetched.metadata.size_bytes.is_some());
    assert!(fetched.metadata.checksum_sha256.is_some());
    assert!(fetched.source_metadata.get("source_url").is_some());

    let t = timely_adapter();
    let err = t
        .fetch_artifact(&IrArtifactFetchRequest {
            source_id: "jp_tdnet_timely_html".into(),
            artifact_key: ucel_core::IrArtifactKey {
                document: IrDocumentKey {
                    source_id: "jp_tdnet_timely_html".into(),
                    source_document_id: "TD20260301001".into(),
                },
                artifact_id: "timely-pdf".into(),
            },
        })
        .expect_err("review-required should not pass without approval");
    assert!(err.to_string().contains("review_required"));
}
