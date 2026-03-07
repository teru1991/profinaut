use crate::ir_inventory::load_ir_inventory;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use ucel_ir::{statutory_adapter, timely_adapter, IrSourceAdapter};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn jp_official_source_ids(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let inv = load_ir_inventory(root)?;
    let mut ids = inv
        .sources
        .into_iter()
        .filter(|s| s.market == "jp")
        .filter(|s| {
            s.source_family == "jp_statutory_disclosure"
                || s.source_family == "jp_timely_disclosure"
        })
        .map(|s| s.source_id)
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

pub fn adapter_map() -> BTreeMap<String, Box<dyn IrSourceAdapter>> {
    let mut map: BTreeMap<String, Box<dyn IrSourceAdapter>> = BTreeMap::new();
    map.insert(
        "edinet_api_documents_v2".into(),
        Box::new(statutory_adapter()),
    );
    map.insert("jp_tdnet_timely_html".into(), Box::new(timely_adapter()));
    map
}
