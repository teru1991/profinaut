use std::fs;
use ucel_testkit::ir_jp_official::repo_root;

#[test]
fn ir_jp_official_docs_drift() {
    let root = repo_root();
    let source_matrix =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_source_matrix.md")).unwrap();
    let fetch_flow =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_fetch_flow.md")).unwrap();
    let identity =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_identity_mapping.md")).unwrap();
    let artifact =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_artifact_matrix.md")).unwrap();
    let access =
        fs::read_to_string(root.join("ucel/docs/ir/jp_official_access_and_politeness.md")).unwrap();

    for term in ["edinet_api_documents_v2", "jp_tdnet_timely_html"] {
        assert!(source_matrix.contains(term));
    }
    for step in [
        "list_documents",
        "fetch_document_detail",
        "list_artifacts",
        "fetch_artifact",
    ] {
        assert!(fetch_flow.contains(step));
    }
    for kind in [
        "jp_edinet_code_like",
        "jp_exchange_code_like",
        "jp_local_code_like",
    ] {
        assert!(identity.contains(kind));
    }
    for kind in ["json", "pdf", "html", "xbrl", "zip"] {
        assert!(artifact.contains(kind));
    }
    for key in [
        "Allowed",
        "ReviewRequired",
        "Blocked",
        "retry_budget",
        "max_attachment_bytes",
    ] {
        assert!(access.contains(key));
    }
}
