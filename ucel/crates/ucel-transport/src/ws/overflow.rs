//! Backpressure overflow policies.
//!
//! This module provides:
//! - `OverflowPolicy`: what to do when an in-memory queue is full.
//! - `Spooler`: a simple spill-to-disk implementation (ndjson), using `ucel-journal` WAL.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use ucel_journal::{FsyncMode, RawRecord, WalWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropMode {
    /// Drop the new item (preserve existing queue contents).
    DropNewest,
    /// Drop oldest items from the lowest-priority queue first.
    /// (Selection is implemented by the queue; this is a directive.)
    DropOldestLowPriority,
}

#[derive(Debug, Clone)]
pub enum OverflowPolicy {
    Drop { mode: DropMode },
    /// Wait for capacity up to `max_wait`. If still full, fallback is applied.
    SlowDown {
        max_wait: Duration,
        fallback: DropMode,
    },
    /// Spill the item to disk; if spill fails, fallback is applied.
    SpillToDisk {
        spooler: Arc<Spooler>,
        fallback: DropMode,
    },
}

impl OverflowPolicy {
    pub fn drop_newest() -> Self {
        Self::Drop {
            mode: DropMode::DropNewest,
        }
    }

    pub fn drop_oldest_low_priority() -> Self {
        Self::Drop {
            mode: DropMode::DropOldestLowPriority,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpoolerConfig {
    pub dir: PathBuf,
    pub max_bytes: u64,
    pub fsync_mode: FsyncMode,
}

impl SpoolerConfig {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            max_bytes: 64 * 1024 * 1024,
            fsync_mode: FsyncMode::Balanced,
        }
    }
}

/// A minimal spill-to-disk writer.
///
/// Uses `ucel-journal::WalWriter` under the hood (ndjson, rotation by size).
pub struct Spooler {
    wal: Mutex<WalWriter>,
}

impl Spooler {
    pub fn open(cfg: SpoolerConfig) -> Result<Self, String> {
        let wal = WalWriter::open(&cfg.dir, cfg.max_bytes, cfg.fsync_mode)?;
        Ok(Self {
            wal: Mutex::new(wal),
        })
    }

    /// Spill bytes to disk as a `RawRecord`.
    ///
    /// IMPORTANT: This is on the overflow path. It should be safe and simple.
    pub async fn spill_bytes(
        &self,
        exchange_id: &str,
        conn_id: &str,
        op_id: &str,
        symbol: Option<&str>,
        kind: &str,
        priority: &str,
        raw: &[u8],
        meta: serde_json::Value,
        ts_unix: u64,
    ) -> Result<(), String> {
        let mut m = serde_json::Map::new();
        m.insert("kind".into(), serde_json::Value::String(kind.to_string()));
        m.insert(
            "priority".into(),
            serde_json::Value::String(priority.to_string()),
        );
        m.insert("meta".into(), meta);
        let rec = RawRecord {
            ts: ts_unix,
            exchange_id: exchange_id.to_string(),
            conn_id: conn_id.to_string(),
            op_id: op_id.to_string(),
            symbol: symbol.map(|s| s.to_string()),
            raw_bytes_b64: base64::engine::general_purpose::STANDARD.encode(raw),
            meta: serde_json::Value::Object(m),
        };

        let mut w = self.wal.lock().await;
        w.append(&rec)
    }
}

impl std::fmt::Debug for Spooler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spooler").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn spooler_writes_files() {
        let dir = tempfile::tempdir().unwrap();
        let sp = Spooler::open(SpoolerConfig {
            dir: dir.path().to_path_buf(),
            max_bytes: 512,
            fsync_mode: FsyncMode::Balanced,
        })
            .unwrap();

        sp.spill_bytes(
            "x",
            "c1",
            "op",
            None,
            "overflow",
            "public",
            b"hello",
            serde_json::json!({"why":"test"}),
            1,
        )
            .await
            .unwrap();

        let files: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert!(!files.is_empty());
    }
}