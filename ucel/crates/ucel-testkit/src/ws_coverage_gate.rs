use crate::{load_coverage_manifest, CoverageGateResult};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn read_supported_ws_ops_from_channels_mod(path: &Path) -> Result<HashSet<String>, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut out = HashSet::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with('"') && line.ends_with(",") {
            out.insert(line.trim_end_matches(',').trim_matches('"').to_string());
        }
    }
    Ok(out)
}

pub fn run_ws_channels_gate(repo_root: &Path) -> Result<Vec<(String, CoverageGateResult)>, String> {
    let coverage_dir = repo_root.join("ucel/coverage");
    let mut out = Vec::new();
    for entry in fs::read_dir(&coverage_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }

        let manifest = load_coverage_manifest(&path).map_err(|e| e.to_string())?;
        let channels_mod = repo_root
            .join("ucel/crates")
            .join(format!("ucel-cex-{}/src/channels/mod.rs", manifest.venue));

        let supported = if channels_mod.exists() {
            read_supported_ws_ops_from_channels_mod(&channels_mod)?
        } else {
            HashSet::new()
        };

        let coverage_ws_ops: Vec<String> = manifest
            .entries
            .iter()
            .filter(|e| e.id.starts_with("crypto.public.ws.") || e.id.starts_with("crypto.private.ws."))
            .map(|e| e.id.clone())
            .collect();

        let missing_ops: Vec<String> = coverage_ws_ops
            .into_iter()
            .filter(|op| !supported.contains(op))
            .collect();

        let mut missing: HashMap<String, Vec<String>> = HashMap::new();
        if !missing_ops.is_empty() {
            missing.insert("ws_ops".into(), missing_ops);
        }

        let result = if missing.is_empty() {
            CoverageGateResult::Passed
        } else if manifest.strict {
            CoverageGateResult::Failed(missing)
        } else {
            CoverageGateResult::WarnOnly(missing)
        };
        out.push((manifest.venue, result));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_gate_executes_against_repo_coverage() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let rs = run_ws_channels_gate(&root).expect("gate run");
        assert!(!rs.is_empty());
        let strict_failures = rs
            .iter()
            .filter(|(_, r)| matches!(r, CoverageGateResult::Failed(_)))
            .count();
        assert_eq!(strict_failures, 0, "strict venues must have full ws op coverage");
    }
}
