use ucel_core::{IrArtifactKey, IrDocumentKey};
use ucel_ir::{jp_issuer_html_adapter, IrArtifactFetchRequest, IrArtifactListRequest, IrSourceAdapter};

#[test]
fn ir_issuer_sites_artifact_download() {
    let adapter = jp_issuer_html_adapter();
    let listed = adapter
        .list_artifacts(&IrArtifactListRequest {
            source_id: "jp_issuer_ir_html_public".into(),
            document_key: IrDocumentKey { source_id: "jp_issuer_ir_html_public".into(), source_document_id: "JPDOC1".into() },
        })
        .expect("list artifacts");
    assert!(!listed.artifacts.is_empty());

    let err = adapter
        .fetch_artifact(&IrArtifactFetchRequest {
            source_id: "jp_issuer_ir_html_public".into(),
            artifact_key: IrArtifactKey { document: IrDocumentKey { source_id: "jp_issuer_ir_html_public".into(), source_document_id: "JPDOC1".into() }, artifact_id: "jpdoc1-html".into() },
        })
        .expect_err("review-required fetch should fail without explicit approval");
    assert!(err.to_string().contains("review_required"));
}
