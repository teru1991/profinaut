use crate::ir_inventory::load_ir_inventory;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use ucel_ir::{sec_adapter, IrSourceAdapter};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn us_official_source_ids(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let inv = load_ir_inventory(root)?;
    let mut ids = inv
        .sources
        .into_iter()
        .filter(|s| s.market == "us")
        .filter(|s| s.source_family == "us_sec_disclosure")
        .map(|s| s.source_id)
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

pub fn adapter_map() -> BTreeMap<String, Box<dyn IrSourceAdapter>> {
    let mut map: BTreeMap<String, Box<dyn IrSourceAdapter>> = BTreeMap::new();
    map.insert("sec_edgar_submissions_api".into(), Box::new(sec_adapter()));
    map
}
