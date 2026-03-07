use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct DomesticPublicInventory {
    pub entries: Vec<InventoryEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InventoryEntry {
    pub venue: String,
    pub api_kind: String,
    pub public_id: String,
    pub surface_class: String,
    pub current_repo_status: String,
}

#[derive(Debug, Deserialize)]
pub struct WsFixtureBundle {
    pub channels: Vec<WsFixtureChannel>,
}

#[derive(Debug, Deserialize)]
pub struct WsFixtureChannel {
    pub venue: String,
    pub public_id: String,
    pub ack_mode: String,
    pub integrity_mode: String,
    pub heartbeat_required: bool,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn load_inventory(root: &Path) -> Result<DomesticPublicInventory, Box<dyn std::error::Error>> {
    let p = root.join("ucel/coverage_v2/domestic_public/jp_public_inventory.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn load_ws_fixture_bundle(root: &Path) -> Result<WsFixtureBundle, Box<dyn std::error::Error>> {
    let p = root.join("ucel/fixtures/domestic_public_ws/cases.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn ws_entries(inv: &DomesticPublicInventory) -> Vec<&InventoryEntry> {
    inv.entries.iter().filter(|e| e.api_kind == "ws").collect()
}
