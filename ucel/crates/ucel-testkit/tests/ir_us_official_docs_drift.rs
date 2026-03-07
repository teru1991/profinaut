use std::fs;
use ucel_testkit::ir_us_official::repo_root;

#[test]
fn ir_us_official_docs_drift() {
    let root = repo_root();
    let source_matrix =
        fs::read_to_string(root.join("ucel/docs/ir/us_official_source_matrix.md")).unwrap();
    let fetch_flow =
        fs::read_to_string(root.join("ucel/docs/ir/us_official_fetch_flow.md")).unwrap();
    let identity =
        fs::read_to_string(root.join("ucel/docs/ir/us_official_identity_mapping.md")).unwrap();
    let artifact =
        fs::read_to_string(root.join("ucel/docs/ir/us_official_artifact_matrix.md")).unwrap();
    let filing =
        fs::read_to_string(root.join("ucel/docs/ir/us_official_filing_family_mapping.md")).unwrap();
    let access = fs::read_to_string(root.join("ucel/docs/ir/us_official_access_and_politeness.md"))
        .unwrap();

    assert!(source_matrix.contains("sec_edgar_submissions_api"));
    for step in [
        "list_documents",
        "fetch_document_detail",
        "list_artifacts",
        "fetch_artifact",
    ] {
        assert!(fetch_flow.contains(step));
    }
    for kind in ["us_cik_like", "ticker_like", "exchange_ticker_like"] {
        assert!(identity.contains(kind));
    }
    for kind in ["json", "html", "ixbrl", "pdf", "xml", "zip"] {
        assert!(artifact.contains(kind));
    }
    for fam in [
        "annual",
        "quarterly",
        "current",
        "proxy",
        "registration-like",
        "insider-like",
        "other",
    ] {
        assert!(filing.contains(fam));
    }
    for key in [
        "Allowed",
        "ReviewRequired",
        "Blocked",
        "user_agent",
        "retry_budget",
        "max_attachment_bytes",
    ] {
        assert!(access.contains(key));
    }
}
