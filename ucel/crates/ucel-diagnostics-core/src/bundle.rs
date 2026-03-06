use crate::manifest::manifest_to_pretty_json;
use ucel_core::BundleManifest;

pub fn to_bundle_json(
    manifest: &BundleManifest,
    payload: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Error> {
    let manifest_json: serde_json::Value = serde_json::from_slice(
        &manifest_to_pretty_json(manifest)
            .map_err(|e| serde_json::Error::io(std::io::Error::other(e.to_string())))?,
    )?;
    Ok(serde_json::json!({
        "manifest": manifest_json,
        "payload": payload,
    }))
}
