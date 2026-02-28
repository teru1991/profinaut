use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(default)]
    ws_channels: Vec<WsSpec>,
}

#[derive(Debug, Deserialize)]
struct WsSpec {
    id: String,
}

fn collect_catalog_exchange_ids(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let exchanges_dir = repo_root.join("docs").join("exchanges");
    let mut out = BTreeSet::new();

    for entry in fs::read_dir(&exchanges_dir).map_err(|e| format!("read_dir docs/exchanges: {e}"))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let ex_id = entry.file_name().to_string_lossy().to_string();
        let catalog_path = entry.path().join("catalog.json");
        if catalog_path.exists() {
            let raw = fs::read_to_string(&catalog_path)
                .map_err(|e| format!("read {catalog_path:?}: {e}"))?;
            // Parse to validate JSON structure; fields are not used beyond this check.
            let _cat: Catalog = serde_json::from_str(&raw)
                .map_err(|e| format!("parse {catalog_path:?}: {e}"))?;
            out.insert(ex_id);
        }
    }
    Ok(out)
}

fn collect_coverage_exchange_ids(ucel_root: &Path) -> Result<BTreeSet<String>, String> {
    let cov_dir = ucel_root.join("coverage");
    let mut out = BTreeSet::new();

    for entry in fs::read_dir(&cov_dir).map_err(|e| format!("read_dir ucel/coverage: {e}"))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(stem) = name.strip_suffix(".yaml") {
            out.insert(stem.to_string());
        }
    }
    Ok(out)
}

/// Gate: catalog ids must be present as coverage yaml.
/// (Connectors are allowed to lag behind coverage; strictness is expressed by `strict:` in coverage.)
pub fn run_ssot_gate(repo_root: &Path) -> Result<(), String> {
    let ucel_root = repo_root.join("ucel");

    let catalog_ids = collect_catalog_exchange_ids(repo_root)?;
    let coverage_ids = collect_coverage_exchange_ids(&ucel_root)?;

    let mut missing = Vec::new();

    for cat in &catalog_ids {
        if !coverage_ids.contains(cat) {
            missing.push(format!("catalog has '{cat}' but ucel/coverage/{cat}.yaml is missing"));
        }
    }

    if !missing.is_empty() {
        return Err(format!("SSOT gate failed:\n- {}", missing.join("\n- ")));
    }
    Ok(())
}
