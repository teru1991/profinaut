use crate::domain::IrEvent;
use crate::errors::{UcelIrError, UcelIrErrorKind};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait RawSink {
    fn put_raw(&self, key: &str, data: &[u8]) -> Result<(), UcelIrError>;
}

pub trait EventSink {
    fn put_event(&self, event: IrEvent) -> Result<bool, UcelIrError>;
}

pub struct FsRawSink {
    root: PathBuf,
}

impl FsRawSink {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, UcelIrError> {
        fs::create_dir_all(root.as_ref())
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Sink, e.to_string()))?;
        Ok(Self {
            root: root.as_ref().to_path_buf(),
        })
    }
}

impl RawSink for FsRawSink {
    fn put_raw(&self, key: &str, data: &[u8]) -> Result<(), UcelIrError> {
        let hash = hex::encode(Sha256::digest(key.as_bytes()));
        let path = self.root.join(format!("{hash}.bin"));
        fs::write(path, data).map_err(|e| UcelIrError::new(UcelIrErrorKind::Sink, e.to_string()))
    }
}

#[derive(Default)]
pub struct MemorySink {
    dedupe: Mutex<HashSet<String>>,
    events: Mutex<Vec<IrEvent>>,
    raw: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemorySink {
    pub fn events_len(&self) -> usize {
        self.events.lock().map(|g| g.len()).unwrap_or(0)
    }
}

impl RawSink for MemorySink {
    fn put_raw(&self, key: &str, data: &[u8]) -> Result<(), UcelIrError> {
        let mut raw = self
            .raw
            .lock()
            .map_err(|_| UcelIrError::new(UcelIrErrorKind::Internal, "raw sink lock poisoned"))?;
        raw.insert(key.to_owned(), data.to_vec());
        Ok(())
    }
}

impl EventSink for MemorySink {
    fn put_event(&self, event: IrEvent) -> Result<bool, UcelIrError> {
        let key = event.dedupe_key();
        let mut dedupe = self.dedupe.lock().map_err(|_| {
            UcelIrError::new(UcelIrErrorKind::Internal, "event dedupe lock poisoned")
        })?;
        if dedupe.contains(&key) {
            return Ok(false);
        }
        dedupe.insert(key);
        drop(dedupe);

        let mut events = self
            .events
            .lock()
            .map_err(|_| UcelIrError::new(UcelIrErrorKind::Internal, "events lock poisoned"))?;
        events.push(event);
        Ok(true)
    }
}
