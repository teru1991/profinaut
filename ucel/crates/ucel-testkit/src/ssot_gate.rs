use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct CatalogMeta {
    exchange: Option<String>,
    ids: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct CoverageFile {
    venue: Option<String>,
    scope: Option<String>,
    strict: Option<bool>,
    implemented: Option<bool>,
    tested: Option<bool>,
    #[serde(default)]
    entries: Vec<CoverageEntry>,
}

#[derive(Debug, Deserialize)]
struct CoverageEntry {
    id: Option<String>,
    kind: Option<String>,
    access: Option<String>,
    implemented: Option<bool>,
    tested: Option<bool>,
    /// "not_supported" means the entry is explicitly out of scope (e.g., FIX protocol stubs).
    /// These entries are excluded from strict=true enforcement, matching the lib.rs policy.
    #[serde(default)]
    support: Option<String>,
}

fn collect_catalog_exchange_ids(repo_root: &Path) -> Result<BTreeMap<String, CatalogMeta>, String> {
    let exchanges_dir = repo_root.join("docs").join("exchanges");
    let mut out = BTreeMap::new();

    for entry in
        fs::read_dir(&exchanges_dir).map_err(|e| format!("read_dir docs/exchanges: {e}"))?
    {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }

        let ex_id = entry.file_name().to_string_lossy().to_string();
        let catalog_path = entry.path().join("catalog.json");
        if !catalog_path.exists() {
            continue;
        }

        let raw =
            fs::read_to_string(&catalog_path).map_err(|e| format!("read {catalog_path:?}: {e}"))?;
        let json: Value =
            serde_json::from_str(&raw).map_err(|e| format!("parse {catalog_path:?}: {e}"))?;

        let exchange = json
            .get("exchange")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        let mut ids = BTreeSet::new();
        collect_ids_from_json(&json, &mut ids);

        out.insert(ex_id, CatalogMeta { exchange, ids });
    }
    Ok(out)
}

fn collect_ids_from_json(node: &Value, out: &mut BTreeSet<String>) {
    match node {
        Value::Object(map) => {
            if let Some(id) = map.get("id").and_then(Value::as_str) {
                out.insert(id.to_owned());
            }
            for value in map.values() {
                collect_ids_from_json(value, out);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_ids_from_json(value, out);
            }
        }
        _ => {}
    }
}

fn collect_coverage_files(ucel_root: &Path) -> Result<BTreeMap<String, CoverageFile>, String> {
    let cov_dir = ucel_root.join("coverage");
    let mut out = BTreeMap::new();

    for entry in fs::read_dir(&cov_dir).map_err(|e| format!("read_dir ucel/coverage: {e}"))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(stem) = name.strip_suffix(".yaml") else {
            continue;
        };

        let path = entry.path();
        let raw = fs::read_to_string(&path).map_err(|e| format!("read {path:?}: {e}"))?;
        let parsed: CoverageFile =
            serde_yaml::from_str(&raw).map_err(|e| format!("parse {path:?}: {e}"))?;
        out.insert(stem.to_string(), parsed);
    }

    Ok(out)
}

fn validate_scope(scope: &str) -> bool {
    matches!(scope, "public_only" | "public_private")
}

fn validate_entry_field(value: Option<&str>, allowed: &[&str]) -> bool {
    value
        .map(|v| allowed.iter().any(|allowed| allowed == &v))
        .unwrap_or(true)
}

/// Gate: catalog ids must be present as coverage yaml and strict venues must be fully green.
///
/// Backward compatibility note:
/// v1 schema fields (scope/implemented/tested/kind/access) are currently treated as optional
/// to avoid breaking existing coverage files. Once all venues are migrated, these become required.
pub fn run_ssot_gate(repo_root: &Path) -> Result<(), String> {
    let ucel_root = repo_root.join("ucel");

    let catalogs = collect_catalog_exchange_ids(repo_root)?;
    let coverages = collect_coverage_files(&ucel_root)?;

    let mut failures = Vec::new();

    for (venue, catalog) in &catalogs {
        let Some(coverage) = coverages.get(venue) else {
            failures.push(format!(
                "venue={venue}: catalog exists but ucel/coverage/{venue}.yaml is missing"
            ));
            continue;
        };

        if let Some(exchange) = &catalog.exchange {
            if exchange != venue {
                failures.push(format!(
                    "venue={venue}: catalog.exchange='{exchange}' must match directory name"
                ));
            }
        }

        if let Some(coverage_venue) = &coverage.venue {
            if coverage_venue != venue {
                failures.push(format!(
                    "venue={venue}: coverage venue field '{coverage_venue}' does not match file stem '{venue}'"
                ));
            }
        }

        if let Some(scope) = &coverage.scope {
            if !validate_scope(scope) {
                failures.push(format!(
                    "venue={venue}: invalid scope '{scope}' (expected public_only/public_private)"
                ));
            }
        }

        let mut coverage_ids = BTreeSet::new();
        for entry in &coverage.entries {
            let Some(id) = entry.id.as_deref() else {
                failures.push(format!("venue={venue}: coverage entry missing id"));
                continue;
            };

            if !catalog.ids.contains(id) {
                failures.push(format!(
                    "venue={venue} id={id}: coverage id not found in catalog.json"
                ));
            }
            coverage_ids.insert(id.to_string());

            if !validate_entry_field(entry.kind.as_deref(), &["rest", "ws"]) {
                failures.push(format!(
                    "venue={venue} id={id}: invalid kind '{:?}'",
                    entry.kind
                ));
            }
            if !validate_entry_field(entry.access.as_deref(), &["public", "private"]) {
                failures.push(format!(
                    "venue={venue} id={id}: invalid access '{:?}'",
                    entry.access
                ));
            }
        }

        // Backward compatible rollout: require full catalog coverage only after v1 scope is declared.
        if coverage.scope.is_some() {
            for catalog_id in &catalog.ids {
                if !coverage_ids.contains(catalog_id) {
                    failures.push(format!(
                        "venue={venue} id={catalog_id}: catalog id missing in coverage.entries"
                    ));
                }
            }
        }

        if coverage.strict.unwrap_or(false) {
            if let Some(false) = coverage.implemented {
                failures.push(format!(
                    "venue={venue}: strict=true requires coverage.implemented=true"
                ));
            }
            if let Some(false) = coverage.tested {
                failures.push(format!(
                    "venue={venue}: strict=true requires coverage.tested=true"
                ));
            }

            for entry in &coverage.entries {
                let Some(id) = entry.id.as_deref() else {
                    continue;
                };
                // Skip entries explicitly marked as out-of-scope (e.g., FIX protocol stubs).
                if entry.support.as_deref() == Some("not_supported") {
                    continue;
                }
                if !entry.implemented.unwrap_or(false) {
                    failures.push(format!(
                        "venue={venue} id={id}: strict=true requires implemented=true"
                    ));
                }
                if !entry.tested.unwrap_or(false) {
                    failures.push(format!(
                        "venue={venue} id={id}: strict=true requires tested=true"
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        return Err(format!("SSOT gate failed:\n- {}", failures.join("\n- ")));
    }
    Ok(())
}

use crate::ssot_integrity_gate::run_ssot_integrity_gate;
use crate::ssot_integrity_gate_types::GateReport;

/// v2: SSOT Integrity Gate (Catalog ↔ Coverage ↔ Crate ↔ Rules ↔ Examples)
/// NOTE: This is a new API; existing v1 gate behavior is intentionally unchanged in Task2.
///       CI enabling is done in Task3 after SSOT data is fully prepared.
pub fn run_ssot_integrity_gate_v2(repo_root: &Path) -> Result<GateReport, String> {
    run_ssot_integrity_gate(repo_root)
}
