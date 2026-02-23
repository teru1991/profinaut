use std::path::Path;

use ucel_subscription_planner::{extract_ws_ops, load_manifest};

pub fn public_crypto_ws_ops_from_coverage(
    coverage_dir: &Path,
    exchange_id: &str,
) -> Result<Vec<String>, String> {
    let manifest_path = coverage_dir.join(format!("{exchange_id}.yaml"));
    let manifest = load_manifest(&manifest_path)?;
    let mut ops = extract_ws_ops(&manifest);
    ops.retain(|op| op.starts_with("crypto.public.ws."));
    Ok(ops)
}
