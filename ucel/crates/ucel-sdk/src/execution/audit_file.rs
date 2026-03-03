use crate::execution::{
    AuditEvent, AuditReplayFilter, AuditSink, SdkExecutionError, SdkExecutionErrorCode,
    SdkExecutionResult,
};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// FileAuditSink の設定
#[derive(Clone)]
pub struct FileAuditSinkConfig {
    /// 書き込み先 ndjson ファイルパス
    pub path: PathBuf,
    /// true にすると append ごとに fsync を行い耐障害性を高める（低速になる）
    pub fsync_each_append: bool,
    /// 1 行あたりの最大バイト数（デフォルト推奨: 1MB）
    pub max_line_bytes: usize,
}

/// 永続監査 ndjson/WAL 実装。
/// - append は追記のみ（既存行を変更しない）
/// - 壊れた行は salvage（スキップ）して replay を継続
/// - secrets を含まない運用を docs で強制（tags に secrets を入れないこと）
pub struct FileAuditSink {
    cfg: FileAuditSinkConfig,
    lock: Mutex<()>,
}

impl FileAuditSink {
    pub fn new(cfg: FileAuditSinkConfig) -> Self {
        Self {
            cfg,
            lock: Mutex::new(()),
        }
    }

    fn open_append(&self) -> SdkExecutionResult<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cfg.path)
            .map_err(|e| {
                SdkExecutionError::new(
                    SdkExecutionErrorCode::AuditFailure,
                    format!("open audit file failed: {}", e),
                )
            })
    }

    fn open_read(&self) -> SdkExecutionResult<File> {
        File::open(&self.cfg.path).map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::ReplayFailure,
                format!("open audit file for read failed: {}", e),
            )
        })
    }

    fn encode_line(event: &AuditEvent) -> SdkExecutionResult<Vec<u8>> {
        serde_json::to_vec(event).map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::AuditFailure,
                format!("audit encode failed: {}", e),
            )
        })
    }

    fn decode_line(line: &str) -> Option<AuditEvent> {
        serde_json::from_str::<AuditEvent>(line).ok()
    }

    fn match_filter(ev: &AuditEvent, f: &AuditReplayFilter) -> bool {
        if let Some(ref run_id) = f.run_id {
            let ok = match ev {
                AuditEvent::OrderRequested { run_id: r, .. } => {
                    r.as_deref() == Some(run_id.as_str())
                }
                AuditEvent::OrderResult { run_id: r, .. } => r.as_deref() == Some(run_id.as_str()),
                AuditEvent::CancelRequested { run_id: r, .. } => {
                    r.as_deref() == Some(run_id.as_str())
                }
                AuditEvent::CancelResult { run_id: r, .. } => r.as_deref() == Some(run_id.as_str()),
                AuditEvent::ReconcileResult { .. } => false,
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

impl AuditSink for FileAuditSink {
    fn append(&self, event: AuditEvent) -> SdkExecutionResult<Option<String>> {
        let _g = self.lock.lock().map_err(|_| {
            SdkExecutionError::new(SdkExecutionErrorCode::AuditFailure, "audit lock poisoned")
        })?;

        let bytes = Self::encode_line(&event)?;
        if bytes.len() > self.cfg.max_line_bytes {
            return Err(SdkExecutionError::new(
                SdkExecutionErrorCode::AuditFailure,
                "audit line too large",
            ));
        }

        let mut f = self.open_append()?;
        f.write_all(&bytes).map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::AuditFailure,
                format!("write failed: {}", e),
            )
        })?;
        f.write_all(b"\n").map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::AuditFailure,
                format!("write newline failed: {}", e),
            )
        })?;
        if self.cfg.fsync_each_append {
            f.sync_data().map_err(|e| {
                SdkExecutionError::new(
                    SdkExecutionErrorCode::AuditFailure,
                    format!("fsync failed: {}", e),
                )
            })?;
        }
        // v1 では event_id は None（行番号を返す仕組みは次バージョンで）
        Ok(None)
    }

    fn replay(&self, filter: AuditReplayFilter) -> SdkExecutionResult<Vec<AuditEvent>> {
        if !self.cfg.path.exists() {
            return Ok(vec![]);
        }
        let f = self.open_read()?;
        let br = BufReader::new(f);
        let mut out = vec![];

        for line in br.lines() {
            let line = match line {
                Ok(v) => v,
                Err(_) => continue, // salvage（壊れた行はスキップ）
            };
            if line.len() > self.cfg.max_line_bytes {
                continue;
            }
            if let Some(ev) = Self::decode_line(&line) {
                if Self::match_filter(&ev, &filter) {
                    out.push(ev);
                }
            }
        }
        Ok(out)
    }
}
