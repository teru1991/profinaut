use crate::errors::{UcelIrError, UcelIrErrorKind};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait CheckpointStore {
    fn get(&self, key: &str) -> Result<Option<String>, UcelIrError>;
    fn set(&self, key: &str, value: &str) -> Result<(), UcelIrError>;
}

pub struct FsCheckpointStore {
    root: PathBuf,
}

impl FsCheckpointStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, UcelIrError> {
        fs::create_dir_all(root.as_ref())
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Checkpoint, e.to_string()))?;
        Ok(Self {
            root: root.as_ref().to_path_buf(),
        })
    }

    fn key_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.checkpoint"))
    }
}

impl CheckpointStore for FsCheckpointStore {
    fn get(&self, key: &str) -> Result<Option<String>, UcelIrError> {
        let path = self.key_path(key);
        if !path.exists() {
            return Ok(None);
        }
        fs::read_to_string(path)
            .map(Some)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Checkpoint, e.to_string()))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), UcelIrError> {
        fs::write(self.key_path(key), value)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Checkpoint, e.to_string()))
    }
}

#[derive(Default)]
pub struct MemoryCheckpointStore {
    state: Mutex<HashMap<String, String>>,
}

impl CheckpointStore for MemoryCheckpointStore {
    fn get(&self, key: &str) -> Result<Option<String>, UcelIrError> {
        let state = self
            .state
            .lock()
            .map_err(|_| UcelIrError::new(UcelIrErrorKind::Internal, "checkpoint lock poisoned"))?;
        Ok(state.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), UcelIrError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| UcelIrError::new(UcelIrErrorKind::Internal, "checkpoint lock poisoned"))?;
        state.insert(key.to_owned(), value.to_owned());
        Ok(())
    }
}
