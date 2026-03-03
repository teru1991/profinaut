use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsCoverageGateResult {
    Passed,
    Warn { venue: String, missing: Vec<String> },
    Failed { venue: String, missing: Vec<String> },
}

fn supported_ops_len_for_venue(repo_root: &Path, venue: &str) -> Result<Option<usize>, String> {
    let path = repo_root
        .join("ucel/crates")
        .join(format!("ucel-cex-{venue}"))
        .join("src/channels/mod.rs");
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(Some(
        raw.lines()
            .filter(|line| {
                let l = line.trim();
                l.starts_with('"') && l.ends_with("\",")
            })
            .count(),
    ))
}

pub fn run_ws_channels_gate(repo_root: &Path) -> Result<Vec<WsCoverageGateResult>, String> {
    let strict = crate::coverage_v2::load_strict_venues(repo_root).map_err(|e| e.to_string())?;
    let mut output = Vec::new();

    for venue in strict.strict_ws_golden {
        let coverage_path = repo_root
            .join("ucel/coverage/coverage_v2/exchanges")
            .join(format!("{venue}.json"));
        let coverage = crate::coverage_v2::load_json(&coverage_path).map_err(|e| e.to_string())?;
        let public_ws = crate::coverage_v2::public_ws(&coverage).map_err(|e| e.to_string())?;
        if !public_ws {
            output.push(WsCoverageGateResult::Passed);
            continue;
        }

        match supported_ops_len_for_venue(repo_root, &venue)? {
            Some(0) => output.push(WsCoverageGateResult::Warn {
                venue,
                missing: vec!["supported_ws_ops()".to_string()],
            }),
            Some(_) => output.push(WsCoverageGateResult::Passed),
            None => output.push(WsCoverageGateResult::Warn {
                venue,
                missing: vec!["channels/mod.rs missing".to_string()],
            }),
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
