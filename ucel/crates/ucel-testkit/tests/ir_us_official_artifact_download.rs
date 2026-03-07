use ucel_core::IrDocumentKey;
use ucel_ir::{sec_adapter, IrArtifactFetchRequest, IrArtifactListRequest, IrSourceAdapter};

#[test]
fn ir_us_official_artifact_download() {
    let a = sec_adapter();
    let list = a
        .list_artifacts(&IrArtifactListRequest {
            source_id: "sec_edgar_submissions_api".into(),
            document_key: IrDocumentKey {
                source_id: "sec_edgar_submissions_api".into(),
                source_document_id: "0000320193-25-000081".into(),
            },
        })
        .expect("list artifacts");
    assert!(!list.artifacts.is_empty());

    let fetched = a
        .fetch_artifact(&IrArtifactFetchRequest {
            source_id: "sec_edgar_submissions_api".into(),
            artifact_key: list.artifacts[0].key.clone(),
        })
        .expect("fetch artifact");
    assert!(fetched.metadata.size_bytes.is_some());
    assert!(fetched.metadata.checksum_sha256.is_some());
    assert!(fetched.source_metadata.get("source_url").is_some());
    assert!(fetched.source_metadata.get("user_agent").is_some());
}
