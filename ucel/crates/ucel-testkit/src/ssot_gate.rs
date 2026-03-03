use serde_json::Value;
use std::fs;
use std::path::Path;

fn load_catalog(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn ws_channels_len(catalog: &Value) -> Option<usize> {
    for key in ["ws_channels", "wsChannels"] {
        if let Some(arr) = catalog.get(key).and_then(Value::as_array) {
            return Some(arr.len());
        }
    }
    None
}

pub fn run_ssot_gate(repo_root: &Path) -> Result<(), String> {
    let strict = crate::coverage_v2::load_strict_venues(repo_root).map_err(|e| e.to_string())?;
    let mut failures = Vec::new();

    for venue in strict.strict_ws_golden {
        let catalog_path = repo_root
            .join("docs")
            .join("exchanges")
            .join(&venue)
            .join("catalog.json");
        if !catalog_path.exists() {
            failures.push(format!(
                "venue={venue}: missing docs/exchanges/{venue}/catalog.json"
            ));
            continue;
        }
        let catalog = load_catalog(&catalog_path)?;

        let coverage_path = repo_root
            .join("ucel/coverage/coverage_v2/exchanges")
            .join(format!("{venue}.json"));
        if !coverage_path.exists() {
            failures.push(format!(
                "venue={venue}: missing ucel/coverage/coverage_v2/exchanges/{venue}.json"
            ));
            continue;
        }
        let coverage = crate::coverage_v2::load_json(&coverage_path).map_err(|e| e.to_string())?;

        if let Some(exchange) = catalog.get("exchange").and_then(Value::as_str) {
            if exchange != venue {
                failures.push(format!(
                    "venue={venue}: catalog.exchange='{exchange}' must match directory name"
                ));
            }
        }

        if crate::coverage_v2::public_ws(&coverage).map_err(|e| e.to_string())?
            && matches!(ws_channels_len(&catalog), Some(0))
        {
            failures.push(format!(
                "venue={venue}: coverage_v2 public.ws=true requires non-empty catalog ws_channels"
            ));
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("SSOT gate failed:\n- {}", failures.join("\n- ")))
    }
}

use crate::ssot_integrity_gate::run_ssot_integrity_gate;
use crate::ssot_integrity_gate_types::GateReport;

pub fn run_ssot_integrity_gate_v2(repo_root: &Path) -> Result<GateReport, String> {
    run_ssot_integrity_gate(repo_root)
}
