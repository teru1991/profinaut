use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRecord {
    pub ts: u64,
    pub exchange_id: String,
    pub conn_id: String,
    pub op_id: String,
    pub symbol: Option<String>,
    pub raw_bytes_b64: String,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone, Copy)]
pub enum FsyncMode {
    SafeEveryRecord,
    SafeEveryN(usize),
    Balanced,
}

pub struct WalWriter {
    dir: PathBuf,
    max_bytes: u64,
    fsync_mode: FsyncMode,
    current_path: PathBuf,
    current_file: File,
    writes_since_sync: usize,
}

impl WalWriter {
    pub fn open(
        dir: impl AsRef<Path>,
        max_bytes: u64,
        fsync_mode: FsyncMode,
    ) -> Result<Self, String> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let current_path = next_wal_path(&dir)?;
        let current_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_path)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            dir,
            max_bytes,
            fsync_mode,
            current_path,
            current_file,
            writes_since_sync: 0,
        })
    }

    pub fn append(&mut self, record: &RawRecord) -> Result<(), String> {
        let line = serde_json::to_vec(record).map_err(|e| e.to_string())?;
        self.current_file
            .write_all(&line)
            .map_err(|e| e.to_string())?;
        self.current_file
            .write_all(b"\n")
            .map_err(|e| e.to_string())?;
        self.writes_since_sync += 1;
        self.maybe_sync()?;
        self.rotate_if_needed()?;
        Ok(())
    }

    fn maybe_sync(&mut self) -> Result<(), String> {
        match self.fsync_mode {
            FsyncMode::SafeEveryRecord => {
                self.current_file.sync_data().map_err(|e| e.to_string())?
            }
            FsyncMode::SafeEveryN(n) if n > 0 && self.writes_since_sync >= n => {
                self.current_file.sync_data().map_err(|e| e.to_string())?;
                self.writes_since_sync = 0;
            }
            FsyncMode::Balanced => {}
            _ => {}
        }
        Ok(())
    }

    fn rotate_if_needed(&mut self) -> Result<(), String> {
        let sz = self
            .current_file
            .metadata()
            .map_err(|e| e.to_string())?
            .len();
        if sz >= self.max_bytes {
            self.current_file.sync_all().map_err(|e| e.to_string())?;
            self.current_path = next_wal_path(&self.dir)?;
            self.current_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.current_path)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn next_wal_path(dir: &Path) -> Result<PathBuf, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    Ok(dir.join(format!("raw-{ts}.ndjson")))
}

pub fn read_records(path: &Path) -> Result<Vec<RawRecord>, String> {
    let f = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<RawRecord>(&line) {
            out.push(v);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_read_rotate_and_partial_tolerance() {
        let dir = tempfile::tempdir().unwrap();
        let mut wal = WalWriter::open(dir.path(), 120, FsyncMode::SafeEveryN(1)).unwrap();
        for i in 0..5 {
            wal.append(&RawRecord {
                ts: i,
                exchange_id: "binance".into(),
                conn_id: "c1".into(),
                op_id: "crypto.public.ws.trade".into(),
                symbol: Some("BTC/USDT".into()),
                raw_bytes_b64: "e30=".into(),
                meta: serde_json::json!({"i": i}),
            })
            .unwrap();
        }
        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|x| x.unwrap().path())
            .collect();
        assert!(!files.is_empty());
        let one = &files[0];
        let recs = read_records(one).unwrap();
        assert!(!recs.is_empty());

        let partial = dir.path().join("partial.ndjson");
        fs::write(&partial, "{\"ts\":1,\"exchange_id\":\"x\",\"conn_id\":\"c\",\"op_id\":\"o\",\"symbol\":null,\"raw_bytes_b64\":\"e30=\",\"meta\":{}}\n{\"broken\"").unwrap();
        let recovered = read_records(&partial).unwrap();
        assert_eq!(recovered.len(), 1);
    }
}
