use crate::{load_coverage_manifest, CoverageGateResult};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn supported_ops_for_exchange(exchange: &str) -> Vec<&'static str> {
    match exchange {
        "binance" => ucel_cex_binance::channels::supported_ws_ops(),
        _ => vec![],
    }
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
        let required: Vec<String> = manifest
            .entries
            .iter()
            .filter(|e| e.id.starts_with("crypto.public.ws.") || e.id.starts_with("crypto.private.ws."))
            .map(|e| e.id.clone())
            .collect();

        let supported: HashSet<String> = supported_ops_for_exchange(&manifest.venue)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let missing_ops: Vec<String> = required
            .into_iter()
            .filter(|op| !supported.contains(op))
            .collect();

        let result = if missing_ops.is_empty() {
            CoverageGateResult::Passed
        } else {
            let mut gaps: HashMap<String, Vec<String>> = HashMap::new();
            gaps.insert("missing_ws_ops".to_string(), missing_ops);
            if manifest.strict {
                CoverageGateResult::Failed(gaps)
            } else {
                CoverageGateResult::WarnOnly(gaps)
            }
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

    #[test]
    fn binance_has_no_missing_crypto_ws_ops() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let rs = run_ws_channels_gate(&root).expect("gate run");
        let binance = rs.into_iter().find(|(v, _)| v == "binance").unwrap();
        assert!(matches!(binance.1, CoverageGateResult::Passed));
    }
}
