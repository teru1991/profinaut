use crate::ir_inventory::load_ir_inventory;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use ucel_ir::{
    jp_issuer_feed_adapter, jp_issuer_html_adapter, us_issuer_feed_adapter, us_issuer_html_adapter,
    IrSourceAdapter,
};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn issuer_site_source_ids(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let inv = load_ir_inventory(root)?;
    let mut ids = inv
        .sources
        .into_iter()
        .filter(|s| s.source_family == "jp_issuer_ir_site" || s.source_family == "us_issuer_ir_site")
        .map(|s| s.source_id)
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

pub fn adapter_map() -> BTreeMap<String, Box<dyn IrSourceAdapter>> {
    let mut map: BTreeMap<String, Box<dyn IrSourceAdapter>> = BTreeMap::new();
    map.insert("jp_issuer_ir_html_public".into(), Box::new(jp_issuer_html_adapter()));
    map.insert("jp_issuer_ir_feed_public".into(), Box::new(jp_issuer_feed_adapter()));
    map.insert("us_issuer_ir_html_public".into(), Box::new(us_issuer_html_adapter()));
    map.insert("us_issuer_ir_feed_public".into(), Box::new(us_issuer_feed_adapter()));
    map
}
