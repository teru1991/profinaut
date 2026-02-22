use std::fs;
use std::path::Path;
use thiserror::Error;
use ucel_symbol_store::RegistrySnapshot;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub fn save_snapshot(path: &Path, snapshot: &RegistrySnapshot) -> Result<(), PersistenceError> {
    let body = serde_json::to_vec(snapshot)?;
    fs::write(path, body)?;
    Ok(())
}

pub fn restore_snapshot(path: &Path) -> Result<Option<RegistrySnapshot>, PersistenceError> {
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read(path)?;
    let snapshot = serde_json::from_slice(&body)?;
    Ok(Some(snapshot))
}
