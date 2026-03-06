use ucel_core::DriftFinding;
use std::fs;
use std::path::Path;

pub fn drift_findings_for_docs(repo_root: &Path, docs_rel: &[&str]) -> Vec<DriftFinding> {
    let mut out = Vec::new();
    for rel in docs_rel {
        let p = repo_root.join(rel);
        if !p.exists() {
            out.push(DriftFinding { level: "error".into(), kind: "path_missing".into(), message: format!("missing: {rel}") });
            continue;
        }
        if p.is_file() {
            scan_file(repo_root, &p, &mut out);
        } else if p.is_dir() {
            scan_dir(repo_root, &p, &mut out);
        }
    }
    out
}

fn scan_dir(root: &Path, dir: &Path, out: &mut Vec<DriftFinding>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() { scan_dir(root, &p, out); }
            else if p.is_file() { scan_file(root, &p, out); }
        }
    }
}

fn scan_file(root: &Path, path: &Path, out: &mut Vec<DriftFinding>) {
    let Ok(txt) = fs::read_to_string(path) else { return; };
    for (idx, _) in txt.match_indices("docs/") {
        let tail = &txt[idx..];
        let end = tail.find(|c: char| c.is_whitespace() || [')',']','\"','\''].contains(&c)).unwrap_or(tail.len());
        let ref_path = &tail[..end];
        if !root.join(ref_path).exists() {
            out.push(DriftFinding { level: "error".into(), kind: "docs_link".into(), message: format!("{} -> missing {}", path.display(), ref_path) });
        }
    }
}
