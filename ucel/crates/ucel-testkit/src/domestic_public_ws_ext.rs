use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct DomesticPublicInventory {
    pub entries: Vec<InventoryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryEntry {
    pub venue: String,
    pub api_kind: String,
    pub public_id: String,
    pub surface_class: String,
    pub current_repo_status: String,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBundle {
    pub cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureCase {
    pub venue: String,
    pub operation_id: String,
    pub source_channel: String,
    pub payload: serde_json::Value,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn load_inventory(root: &Path) -> Result<DomesticPublicInventory, Box<dyn std::error::Error>> {
    let p = root.join("ucel/coverage_v2/domestic_public/jp_public_inventory.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn load_fixtures(root: &Path) -> Result<FixtureBundle, Box<dyn std::error::Error>> {
    let p = root.join("ucel/fixtures/domestic_public_ext_ws/cases.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn vendor_ws_entries(inv: &DomesticPublicInventory) -> Vec<&InventoryEntry> {
    inv.entries
        .iter()
        .filter(|e| e.api_kind == "ws" && e.surface_class == "vendor_public_extension")
        .collect()
}
