use crate::ssot_integrity_gate_types::{GateIssue, GateReport};
use crate::{CoverageEntry, CoverageManifest, CoverageSupport};

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const CODE_CATALOG_MISSING: &str = "CATALOG_MISSING";
const CODE_COVERAGE_MISSING_FILE: &str = "COVERAGE_MISSING_FILE";
const CODE_COVERAGE_MISSING_ENTRY: &str = "COVERAGE_MISSING_ENTRY";
const CODE_COVERAGE_EXTRA_ENTRY: &str = "COVERAGE_EXTRA_ENTRY";
const CODE_CRATE_MISSING: &str = "CRATE_MISSING";
const CODE_RULES_MISSING: &str = "RULES_MISSING";
const CODE_EXAMPLE_MISSING: &str = "EXAMPLE_MISSING";
const CODE_NOT_SUPPORTED_BUT_IMPL: &str = "NOT_SUPPORTED_BUT_IMPL_OR_TESTED";
const CODE_STRICT_NOT_SUPPORTED: &str = "STRICT_NOT_SUPPORTED";

#[derive(Debug, Clone)]
pub struct VenueCatalog {
    pub venue: String,
    pub ws_op_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct VenueCoverage {
    pub venue: String,
    pub manifest: CoverageManifest,
    pub entries_by_id: HashMap<String, CoverageEntry>,
}

pub fn run_ssot_integrity_gate(repo_root: &Path) -> Result<GateReport, String> {
    let mut report = GateReport::default();

    let catalogs = load_all_catalogs(repo_root, &mut report)?;
    let coverages = load_all_coverages(repo_root, &catalogs, &mut report)?;

    for (venue, cat) in &catalogs {
        let cov = match coverages.get(venue) {
            Some(v) => v,
            None => {
                continue;
            }
        };

        for op_id in &cat.ws_op_ids {
            if !cov.entries_by_id.contains_key(op_id) {
                report.push(
                    GateIssue::fail(
                        CODE_COVERAGE_MISSING_ENTRY,
                        "catalog op id is missing in coverage entries (NOT SUPPORTED must be explicit)",
                    )
                    .with_ctx("venue", venue.clone())
                    .with_ctx("op_id", op_id.clone()),
                );
            }
        }

        for op_id in cov.entries_by_id.keys() {
            if !cat.ws_op_ids.contains(op_id) {
                report.push(
                    GateIssue::warn(
                        CODE_COVERAGE_EXTRA_ENTRY,
                        "coverage has an entry not present in catalog (legacy or stale?)",
                    )
                    .with_ctx("venue", venue.clone())
                    .with_ctx("op_id", op_id.clone()),
                );
            }
        }

        for entry in cov.entries_by_id.values() {
            let eff_strict = entry.effective_strict(cov.manifest.strict);
            let supported = entry.support == CoverageSupport::Supported;
            if eff_strict && !supported {
                report.push(
                    GateIssue::fail(
                        CODE_STRICT_NOT_SUPPORTED,
                        "entry is not_supported but effective_strict=true (forbidden; set entry.strict=false or support=supported)",
                    )
                    .with_ctx("venue", venue.clone())
                    .with_ctx("op_id", entry.id.clone()),
                );
            }

            if !supported && (entry.implemented || entry.tested) {
                report.push(
                    GateIssue::warn(
                        CODE_NOT_SUPPORTED_BUT_IMPL,
                        "entry is not_supported but implemented/tested is true (inconsistent)",
                    )
                    .with_ctx("venue", venue.clone())
                    .with_ctx("op_id", entry.id.clone())
                    .with_ctx("implemented", entry.implemented.to_string())
                    .with_ctx("tested", entry.tested.to_string()),
                );
            }
        }

        if !venue_crate_exists(repo_root, venue) {
            report.push(
                GateIssue::fail(
                    CODE_CRATE_MISSING,
                    "venue crate directory does not exist (expected ucel/crates/ucel-cex-<venue>)",
                )
                .with_ctx("venue", venue.clone())
                .with_ctx("expected_path", format!("ucel/crates/ucel-cex-{venue}")),
            );
        }

        let rules_ok = venue_rules_exist(repo_root, venue);
        if !rules_ok {
            let issue = if cov.manifest.strict {
                GateIssue::fail(
                    CODE_RULES_MISSING,
                    "strict venue requires at least one ws-rules file",
                )
            } else {
                GateIssue::warn(
                    CODE_RULES_MISSING,
                    "non-strict venue missing ws-rules file (recommended)",
                )
            };
            report.push(issue.with_ctx("venue", venue.clone()));
        }

        let example_ok = venue_example_exists(repo_root, venue);
        if !example_ok {
            let issue = if cov.manifest.strict {
                GateIssue::fail(
                    CODE_EXAMPLE_MISSING,
                    "strict venue requires venue smoke example",
                )
            } else {
                GateIssue::warn(
                    CODE_EXAMPLE_MISSING,
                    "non-strict venue missing venue smoke example (recommended)",
                )
            };
            report.push(issue.with_ctx("venue", venue.clone()));
        }
    }

    Ok(report)
}

fn load_all_catalogs(
    repo_root: &Path,
    report: &mut GateReport,
) -> Result<HashMap<String, VenueCatalog>, String> {
    let base = repo_root.join("docs").join("exchanges");
    if !base.exists() {
        report.push(GateIssue::fail(
            CODE_CATALOG_MISSING,
            "docs/exchanges directory is missing",
        ));
        return Ok(HashMap::new());
    }

    let mut out: HashMap<String, VenueCatalog> = HashMap::new();
    let dirs = read_dir_dirs(&base)?;

    for venue_dir in dirs {
        let venue = venue_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if venue.is_empty() {
            continue;
        }

        let catalog_path = venue_dir.join("catalog.json");
        if !catalog_path.exists() {
            continue;
        }

        let json = fs::read_to_string(&catalog_path)
            .map_err(|e| format!("failed to read catalog {}: {}", catalog_path.display(), e))?;
        let v: Value = serde_json::from_str(&json)
            .map_err(|e| format!("failed to parse catalog {}: {}", catalog_path.display(), e))?;

        let ws_op_ids = extract_ws_op_ids_from_catalog(&v);

        out.insert(venue.clone(), VenueCatalog { venue, ws_op_ids });
    }

    Ok(out)
}

fn extract_ws_op_ids_from_catalog(v: &Value) -> HashSet<String> {
    let mut out = HashSet::new();

    let candidates = ["ws_channels", "wsChannels", "ws_channel", "wsChannel"];

    for key in candidates {
        if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
            for item in arr {
                if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                    out.insert(id.to_string());
                }
            }
        }
    }

    out
}

fn load_all_coverages(
    repo_root: &Path,
    catalogs: &HashMap<String, VenueCatalog>,
    report: &mut GateReport,
) -> Result<HashMap<String, VenueCoverage>, String> {
    let base = repo_root
        .join("ucel")
        .join("coverage")
        .join("coverage_v2")
        .join("exchanges");
    let strict = crate::coverage_v2::load_strict_venues(repo_root)
        .map_err(|e| format!("load strict_venues.json: {e}"))?;
    let strict_set: HashSet<String> = strict.strict_ws_golden.into_iter().collect();

    let mut out: HashMap<String, VenueCoverage> = HashMap::new();

    for venue in catalogs.keys() {
        let path = base.join(format!("{venue}.json"));
        if !path.exists() {
            report.push(
                GateIssue::fail(
                    CODE_COVERAGE_MISSING_FILE,
                    "coverage_v2 json missing for venue",
                )
                .with_ctx("venue", venue.clone())
                .with_ctx("expected_path", path.display().to_string()),
            );
            continue;
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read coverage {}: {}", path.display(), e))?;
        let v: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("failed to parse coverage {}: {}", path.display(), e))?;

        let entries: Vec<CoverageEntry> = v
            .get("ws_ops")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|x| x.as_str())
            .map(|id| CoverageEntry {
                id: id.to_string(),
                implemented: true,
                tested: true,
                support: CoverageSupport::Supported,
                strict: None,
            })
            .collect();

        let manifest = CoverageManifest {
            venue: venue.clone(),
            strict: strict_set.contains(venue),
            entries,
        };

        let mut entries_by_id = HashMap::new();
        for e in &manifest.entries {
            entries_by_id.insert(e.id.clone(), e.clone());
        }

        out.insert(
            venue.clone(),
            VenueCoverage {
                venue: venue.clone(),
                manifest,
                entries_by_id,
            },
        );
    }

    if catalogs.is_empty() {
        report.push(GateIssue::warn(
            CODE_CATALOG_MISSING,
            "no exchange catalogs found under docs/exchanges",
        ));
    }

    Ok(out)
}

fn venue_crate_exists(repo_root: &Path, venue: &str) -> bool {
    let path = repo_root
        .join("ucel")
        .join("crates")
        .join(format!("ucel-cex-{venue}"));
    path.is_dir()
}

fn venue_rules_exist(repo_root: &Path, venue: &str) -> bool {
    let rules_dir = repo_root
        .join("ucel")
        .join("crates")
        .join("ucel-ws-rules")
        .join("rules");
    if !rules_dir.is_dir() {
        return false;
    }

    let exact = format!("{venue}.toml");

    let mut ok = false;
    if let Ok(rd) = fs::read_dir(&rules_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_file() {
                continue;
            }
            let name = match p.file_name().and_then(|n| n.to_str()) {
                Some(s) => s,
                None => continue,
            };

            if name == exact {
                ok = true;
                break;
            }
            if name.starts_with(&format!("{venue}-")) && name.ends_with(".toml") {
                ok = true;
                break;
            }
        }
    }
    ok
}

fn venue_example_exists(repo_root: &Path, venue: &str) -> bool {
    let p = repo_root
        .join("ucel")
        .join("examples")
        .join("venue_smoke")
        .join(format!("{venue}.rs"));
    p.is_file()
}

fn read_dir_dirs(base: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let rd =
        fs::read_dir(base).map_err(|e| format!("failed to read dir {}: {}", base.display(), e))?;
    for ent in rd {
        let ent =
            ent.map_err(|e| format!("failed to read dir entry in {}: {}", base.display(), e))?;
        let path = ent.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    Ok(out)
}
