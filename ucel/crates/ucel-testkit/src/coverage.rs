use std::path::Path;

use ucel_subscription_planner::{extract_ws_ops, load_manifest};

#[deprecated(note = "legacy v1 coverage yaml API; use venues_with_public_ws_v2")]
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

pub fn venues_with_public_ws_v2(repo_root: &Path) -> Result<Vec<String>, String> {
    let root = crate::coverage_v2::locate_coverage_v2_root(repo_root).map_err(|e| e.to_string())?;
    let files = crate::coverage_v2::list_exchange_jsons(&root).map_err(|e| e.to_string())?;
    let mut out = vec![];
    for f in files {
        let v = crate::coverage_v2::load_json(&f).map_err(|e| e.to_string())?;
        let id = crate::coverage_v2::infer_exchange_id(&f, &v);
        if crate::coverage_v2::public_ws(&v).map_err(|e| e.to_string())? {
            out.push(id);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
