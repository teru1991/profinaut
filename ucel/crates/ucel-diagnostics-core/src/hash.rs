use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn hash_paths(repo_root: &Path, rel_paths: &[&str]) -> Result<String, HashError> {
    let mut files = Vec::new();
    for rel in rel_paths {
        let p = repo_root.join(rel);
        if p.is_dir() {
            collect_files(&p, &mut files)?;
        } else if p.is_file() {
            files.push(p);
        }
    }
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        let rel = file.strip_prefix(repo_root).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        hasher.update(rel.as_bytes());
        hasher.update(b"\n");
        let mut bytes = fs::read(&file)?;
        normalize_newlines(&mut bytes);
        hasher.update(&bytes);
        hasher.update(b"\n");
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn default_hash_set(repo_root: &Path) -> Result<ucel_core::BundleHashSet, HashError> {
    Ok(ucel_core::BundleHashSet {
        coverage_hash: hash_paths_existing(repo_root, &["ucel/coverage", "coverage"] )?,
        coverage_v2_hash: hash_paths_existing(repo_root, &["ucel/coverage_v2", "coverage_v2"] )?,
        ws_rules_hash: hash_paths_existing(repo_root, &["ucel/crates/ucel-ws-rules", "crates/ucel-ws-rules", "docs/specs/ucel/ws_ingest_runtime_v1.md"] )?,
        catalog_hash: hash_paths_existing(repo_root, &["docs/exchanges", "ucel/docs/exchanges", "docs/specs/ucel"] )?,
        policy_hash: hash_paths_existing(repo_root, &["ucel/docs/policies", "docs/policies", "docs/specs/ucel"] )?,
        symbol_meta_hash: hash_paths_existing(repo_root, &["docs/specs/ucel/symbol_meta_surface_v1.md", "ucel/crates/ucel-symbol-core", "crates/ucel-symbol-core"] )?,
        execution_surface_hash: hash_paths_existing(repo_root, &["docs/specs/ucel/execution_surface_v1.md", "ucel/crates/ucel-execution-core", "crates/ucel-execution-core"] )?,
        runtime_capability_hash: hash_paths_existing(repo_root, &["ucel/crates/ucel-sdk/src", "crates/ucel-sdk/src", "ucel/crates/ucel-registry/src", "crates/ucel-registry/src"] )?,
    })
}

fn hash_paths_existing(repo_root: &Path, candidates: &[&str]) -> Result<String, HashError> {
    let existing: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|rel| repo_root.join(rel).exists())
        .collect();
    hash_paths(repo_root, &existing)
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_files(&p, out)?;
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(())
}

fn normalize_newlines(bytes: &mut Vec<u8>) {
    let s = String::from_utf8_lossy(bytes).replace("\r\n", "\n").replace('\r', "\n");
    *bytes = s.into_bytes();
}
