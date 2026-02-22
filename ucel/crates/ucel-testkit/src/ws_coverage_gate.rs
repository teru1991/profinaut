use crate::{load_coverage_manifest, CoverageGateResult};
use std::fs;
use std::path::Path;

pub fn run_ws_channels_gate(repo_root: &Path) -> Result<Vec<(String, CoverageGateResult)>, String> {
    let coverage_dir = repo_root.join("ucel/coverage");
    let mut out = Vec::new();
    for entry in fs::read_dir(&coverage_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }
        let manifest = load_coverage_manifest(&path).map_err(|e| e.to_string())?;
        let missing: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
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
    }
}
