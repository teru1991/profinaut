use std::fs;
use std::path::PathBuf;

pub fn run(input: &str) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let (_summary, compat, drift) = ucel_diagnostics_analyzer::analyze_support_bundle_value(&value, &repo_root)
        .map_err(|e| e.to_string())?;
    let status = compat.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown");
    if status == "Unsupported" {
        return Err("unsupported diagnostics semver".into());
    }
    let errs = drift
        .get("findings")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter(|f| f.get("level").and_then(|v| v.as_str()) == Some("error")).count())
        .unwrap_or(0);
    if errs > 0 {
        return Err(format!("runbook drift errors detected: {errs}"));
    }
    Ok(())
}
