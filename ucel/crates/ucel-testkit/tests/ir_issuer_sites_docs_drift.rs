use std::fs;
use ucel_testkit::ir_issuer_sites::repo_root;

#[test]
fn ir_issuer_sites_docs_drift() {
    let root = repo_root();
    let matrix = fs::read_to_string(root.join("ucel/docs/ir/jp_us_issuer_site_matrix.md")).unwrap();
    let flow = fs::read_to_string(root.join("ucel/docs/ir/issuer_site_discovery_flow.md")).unwrap();
    let profile = fs::read_to_string(root.join("ucel/docs/ir/issuer_site_profile_guide.md")).unwrap();
    let artifact = fs::read_to_string(root.join("ucel/docs/ir/issuer_site_artifact_matrix.md")).unwrap();
    let access = fs::read_to_string(root.join("ucel/docs/ir/issuer_site_access_and_politeness.md")).unwrap();

    for id in [
        "jp_issuer_ir_html_public",
        "jp_issuer_ir_feed_public",
        "us_issuer_ir_html_public",
        "us_issuer_ir_feed_public",
    ] {
        assert!(matrix.contains(id));
    }
    for step in ["official metadata seed", "inventory seed", "deterministic root traversal"] {
        assert!(flow.contains(step));
    }
    for term in ["IrTop", "NewsArchive", "PresentationLibrary", "page budget"] {
        assert!(profile.contains(term));
    }
    for kind in ["html", "pdf", "xbrl", "ixbrl", "rss"] {
        assert!(artifact.contains(kind));
    }
    for key in ["Allowed", "ReviewRequired", "Blocked", "crawl_depth_cap", "page_budget"] {
        assert!(access.contains(key));
    }
}
