use std::{fs, path::PathBuf};

#[test]
fn runbooks_contract_links_must_exist() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let runbooks_dir = repo_root.join("docs/runbooks");

    let runbook_files = collect_markdown_files(&runbooks_dir);
    assert!(!runbook_files.is_empty(), "no runbook markdown files found");

    for file in runbook_files {
        let content = fs::read_to_string(&file).expect("read runbook file");

        for contract in extract_contract_paths(&content) {
            let contract_path = repo_root.join(&contract);
            assert!(
                contract_path.exists(),
                "missing contract reference {} in {}",
                contract,
                file.display()
            );
        }

        let file_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if file_name.contains("support_bundle") {
            assert!(
                content.contains("bundle") || content.contains("support"),
                "support-bundle runbook must mention bundle context: {}",
                file.display()
            );
        }
    }
}

fn extract_contract_paths(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let needle = "docs/contracts/";

    for (idx, _) in content.match_indices(needle) {
        let tail = &content[idx..];
        let mut end = 0;
        for ch in tail.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-') {
                end += ch.len_utf8();
            } else {
                break;
            }
        }
        if end > 0 {
            let candidate = &tail[..end];
            if candidate.ends_with(".json") {
                links.push(candidate.to_string());
            }
        }
    }

    links.sort();
    links.dedup();
    links
}

fn collect_markdown_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = fs::read_dir(root).expect("read runbooks dir");
    for entry in entries {
        let path = entry.expect("read entry").path();
        if path.is_dir() {
            out.extend(collect_markdown_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    out
}
