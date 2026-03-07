use std::fs;
use ucel_testkit::ir_canonical::repo_root;

#[test]
fn ir_docs_drift_canonical() {
    let root = repo_root();
    let model = fs::read_to_string(root.join("ucel/docs/ir/ir_canonical_model.md")).unwrap();
    for term in [
        "IrSourceFamily",
        "IrIssuerIdentityKind",
        "IrDocumentFamily",
        "IrArtifactKind",
    ] {
        assert!(model.contains(term), "missing term {term}");
    }

    let flow = fs::read_to_string(root.join("ucel/docs/ir/ir_fetch_flow.md")).unwrap();
    for step in [
        "discover_issuers",
        "resolve_issuer",
        "list_documents",
        "fetch_document_detail",
        "list_artifacts",
        "fetch_artifact",
    ] {
        assert!(flow.contains(step), "missing flow step {step}");
    }

    let guard = fs::read_to_string(root.join("ucel/docs/ir/ir_access_guard_policy.md")).unwrap();
    for cls in ["Allowed", "ReviewRequired", "Blocked"] {
        assert!(guard.contains(cls), "missing decision class {cls}");
    }
}
