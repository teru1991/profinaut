use crate::state::DurableStateStore;
use std::fs;
use std::path::Path;

pub fn persist_to_path(store: &DurableStateStore, path: &Path) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn load_from_path(path: &Path) -> Result<DurableStateStore, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&data).map_err(|e| e.to_string())
}
