use crate::load_coverage_manifest;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsCoverageGateResult {
    Passed,
    Warn { venue: String, missing: Vec<String> },
    Failed { venue: String, missing: Vec<String> },
}

fn parse_supported_ops_from_channels_mod(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .filter_map(|line| {
            let l = line.trim();
            if l.starts_with('"') {
                let t = l.trim_end_matches(',').trim();
                if t.ends_with('"') {
                    Some(t.trim_matches('"').to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn supported_ops_for_venue(repo_root: &Path, venue: &str) -> Result<BTreeSet<String>, String> {
    let path = repo_root
        .join("ucel/crates")
        .join(format!("ucel-cex-{venue}"))
        .join("src/channels/mod.rs");
    let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(parse_supported_ops_from_channels_mod(&raw))
}

pub fn run_ws_channels_gate(repo_root: &Path) -> Result<Vec<WsCoverageGateResult>, String> {
    let coverage_dir = repo_root.join("ucel/coverage");
    let mut output = Vec::new();

    for entry in fs::read_dir(&coverage_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }
        let manifest = load_coverage_manifest(&path).map_err(|e| e.to_string())?;
        let expected: BTreeSet<String> = manifest
            .entries
            .iter()
            .map(|e| e.id.as_str())
            .filter(|id| {
                id.starts_with("crypto.public.ws.") || id.starts_with("crypto.private.ws.")
            })
            .map(ToOwned::to_owned)
            .collect();

        let supported = supported_ops_for_venue(repo_root, &manifest.venue)?;
        let missing: Vec<String> = expected.difference(&supported).cloned().collect();

        if missing.is_empty() {
            output.push(WsCoverageGateResult::Passed);
        } else if manifest.strict {
            output.push(WsCoverageGateResult::Failed {
                venue: manifest.venue,
                missing,
            });
        } else {
            output.push(WsCoverageGateResult::Warn {
                venue: manifest.venue,
                missing,
            });
        }
    }

    Ok(output)
}

pub fn summarize_failures(results: &[WsCoverageGateResult]) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    for r in results {
        if let WsCoverageGateResult::Failed { venue, missing } = r {
            map.insert(venue.clone(), missing.clone());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_gate_has_no_strict_failures() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let rs = run_ws_channels_gate(&root).expect("gate run");
        let failures = summarize_failures(&rs);
        assert!(
            failures.is_empty(),
            "strict ws coverage failures: {failures:?}"
        );
    }
}
