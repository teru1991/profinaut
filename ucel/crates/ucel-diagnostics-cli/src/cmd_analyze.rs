use std::fs;
use std::path::PathBuf;

pub fn run(input: &str, output: &str) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let (summary, compat, drift) = ucel_diagnostics_analyzer::analyze_support_bundle_value(&value, &repo_root)
        .map_err(|e| e.to_string())?;
    let out = serde_json::json!({"summary": summary, "compatibility_report": compat, "drift_report": drift});
    fs::write(output, serde_json::to_vec_pretty(&out).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
