//! D2 — Durable append-only spool with segment rotation, total-cap, and
//! on-full policy.
//!
//! ## Segment format
//!
//! Each segment file is named `spool_{seq:06}.dat`.  Records are stored as
//! length-prefix-encoded JSON:
//!
//! ```text
//! [u32 LE: length of JSON bytes][JSON bytes of Envelope]
//! ```
//!
//! ## Crash safety
//!
//! On `open()`, the current write segment is scanned from the beginning.
//! Any record whose header+body cannot be fully read is considered a partial
//! write; the file is **truncated** to the byte offset of the last complete
//! record.  Subsequent writes append to that clean position.
//!
//! ## Segment rotation
//!
//! A new segment is started when the current segment reaches `max_segment_bytes`.
//! Old segments are preserved for the replay worker.
//!
//! ## Total cap
//!
//! `on_full` is enforced when `total_bytes >= max_total_bytes`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

use super::envelope::Envelope;
use super::metrics::PersistenceMetrics;
use super::sink::SinkError;

// ---------------------------------------------------------------------------
// On-full policy
// ---------------------------------------------------------------------------

/// What to do when the spool has reached its total capacity cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnFullPolicy {
    /// Drop ticker-depth (orderbook) envelopes; keep trade envelopes.
    DropTickerDepthKeepTrade,
    /// Drop the entire batch without error.
    DropAll,
    /// Busy-wait with short sleep until space becomes available (blocking).
    Block,
}

impl OnFullPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DropTickerDepthKeepTrade => "drop_ticker_depth_keep_trade",
            Self::DropAll => "drop_all",
            Self::Block => "block",
        }
    }
}

// ---------------------------------------------------------------------------
// Spool configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SpoolConfig {
    pub dir: PathBuf,
    /// Maximum size of a single segment in bytes before rotation.
    pub max_segment_bytes: u64,
    /// Maximum total spool size in bytes across all segments.
    pub max_total_bytes: u64,
    pub on_full: OnFullPolicy,
}

impl SpoolConfig {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            max_segment_bytes: 64 * 1024 * 1024, // 64 MiB
            max_total_bytes: 1024 * 1024 * 1024,  // 1 GiB
            on_full: OnFullPolicy::DropTickerDepthKeepTrade,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal write head
// ---------------------------------------------------------------------------

struct WriteHead {
    seq: u64,
    file: File,
    file_bytes: u64,
}

// ---------------------------------------------------------------------------
// DurableSpool
// ---------------------------------------------------------------------------

/// Durable, append-only spool backed by segment files on disk.
pub struct DurableSpool {
    config: SpoolConfig,
    /// Guarded write head (current segment + position).
    head: Mutex<Option<WriteHead>>,
    /// Sequence number of the current write segment (also in head, but
    /// accessible without the lock for the replay worker).
    current_seq: Arc<std::sync::atomic::AtomicU64>,
    metrics: Arc<PersistenceMetrics>,
}

impl DurableSpool {
    /// Open (or create) the spool.  Creates the directory if needed.
    /// Scans the latest segment for partial records and truncates them.
    pub async fn open(
        config: SpoolConfig,
        metrics: Arc<PersistenceMetrics>,
    ) -> Result<Arc<Self>, SinkError> {
        fs::create_dir_all(&config.dir).await?;

        let mut segments = list_segments(&config.dir).await?;
        segments.sort();

        let (seq, head) = if let Some(last_seg) = segments.last() {
            let path = segment_path(&config.dir, *last_seg);
            let (file, file_bytes) = recover_segment(&path).await?;
            (*last_seg, Some(WriteHead { seq: *last_seg, file, file_bytes }))
        } else {
            (0, None)
        };

        let spool = Arc::new(Self {
            config,
            head: Mutex::new(head),
            current_seq: Arc::new(std::sync::atomic::AtomicU64::new(seq)),
            metrics: metrics.clone(),
        });

        // Refresh metrics from disk.
        spool.refresh_metrics().await?;

        Ok(spool)
    }

    /// Append a batch of envelopes to the spool, applying on_full policy as
    /// needed.  Returns the number of envelopes actually written.
    pub async fn append_batch(
        &self,
        batch: Vec<Envelope>,
    ) -> Result<usize, SinkError> {
        if batch.is_empty() {
            return Ok(0);
        }

        // Serialise all records first to know sizes.
        let records: Vec<Vec<u8>> = batch
            .iter()
            .map(|e| {
                serde_json::to_vec(e).map_err(|e| SinkError::Serialise(e.to_string()))
            })
            .collect::<Result<_, _>>()?;

        let mut written = 0usize;

        for (envelope, record) in batch.iter().zip(records.iter()) {
            let frame_size = 4 + record.len() as u64;

            loop {
                // Check total cap.
                let total_bytes = self.metrics.spool_bytes.load(std::sync::atomic::Ordering::Relaxed);
                if total_bytes + frame_size as i64 > self.config.max_total_bytes as i64 {
                    match self.config.on_full {
                        OnFullPolicy::DropAll => {
                            self.metrics.increment_spool_dropped(&envelope.exchange, &envelope.channel);
                            break; // skip this envelope
                        }
                        OnFullPolicy::DropTickerDepthKeepTrade => {
                            if envelope.channel == "orderbook"
                                || envelope.channel == "depth"
                                || envelope.channel == "ticker"
                            {
                                self.metrics.increment_spool_dropped(&envelope.exchange, &envelope.channel);
                                break;
                            }
                            // Trade channels: fall through to Block behaviour
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            continue; // retry check
                        }
                        OnFullPolicy::Block => {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            continue; // retry check
                        }
                    }
                }

                // Write the record.
                self.write_record(record, frame_size).await?;
                written += 1;
                break;
            }
        }

        Ok(written)
    }

    /// Write a single length-prefix-encoded record, rotating segment if needed.
    async fn write_record(&self, record: &[u8], frame_size: u64) -> Result<(), SinkError> {
        let mut head_guard = self.head.lock().await;

        // Ensure we have an open write head.
        if head_guard.is_none() {
            let seq = 1u64;
            let path = segment_path(&self.config.dir, seq);
            let file = open_segment_for_append(&path).await?;
            self.current_seq.store(seq, std::sync::atomic::Ordering::Release);
            *head_guard = Some(WriteHead { seq, file, file_bytes: 0 });
        }

        let head = head_guard.as_mut().unwrap();

        // Rotate if this record would exceed the segment cap.
        // Never rotate a fresh (empty) segment: a record must always be accepted
        // even if its size alone exceeds max_segment_bytes.
        if head.file_bytes > 0 && head.file_bytes + frame_size > self.config.max_segment_bytes {
            head.file.flush().await?;
            let next_seq = head.seq + 1;
            let path = segment_path(&self.config.dir, next_seq);
            let file = open_segment_for_append(&path).await?;
            self.current_seq
                .store(next_seq, std::sync::atomic::Ordering::Release);
            *head = WriteHead { seq: next_seq, file, file_bytes: 0 };
            self.metrics.spool_segments.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        let h = head_guard.as_mut().unwrap();

        // Write: [u32 LE length][JSON bytes]
        let len = record.len() as u32;
        h.file.write_all(&len.to_le_bytes()).await?;
        h.file.write_all(record).await?;
        h.file.flush().await?;

        h.file_bytes += frame_size;
        self.metrics.add_spool_bytes(frame_size as i64);

        Ok(())
    }

    /// List all complete segments (not the current write segment).
    pub async fn complete_segments(&self) -> Result<Vec<u64>, SinkError> {
        let current = self.current_seq.load(std::sync::atomic::Ordering::Acquire);
        let mut seqs = list_segments(&self.config.dir).await?;
        seqs.retain(|&s| s != current && s != 0);
        seqs.sort();
        Ok(seqs)
    }

    /// Read all envelopes from a segment file.
    pub async fn read_segment(&self, seq: u64) -> Result<Vec<Envelope>, SinkError> {
        let path = segment_path(&self.config.dir, seq);
        read_all_records(&path).await
    }

    /// Delete a segment file after successful replay.
    pub async fn delete_segment(&self, seq: u64) -> Result<(), SinkError> {
        let path = segment_path(&self.config.dir, seq);
        let meta = fs::metadata(&path).await?;
        let file_bytes = meta.len() as i64;
        fs::remove_file(&path).await?;
        self.metrics.add_spool_bytes(-file_bytes);
        self.refresh_metrics().await?;
        Ok(())
    }

    /// Recompute spool_bytes and spool_segments from the actual directory.
    pub async fn refresh_metrics(&self) -> Result<(), SinkError> {
        let seqs = list_segments(&self.config.dir).await?;
        let mut total: i64 = 0;
        for seq in &seqs {
            let path = segment_path(&self.config.dir, *seq);
            if let Ok(meta) = fs::metadata(&path).await {
                total += meta.len() as i64;
            }
        }
        self.metrics.set_spool_bytes(total);
        self.metrics.set_spool_segments(seqs.len() as u64);
        Ok(())
    }

    pub fn config(&self) -> &SpoolConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the path of a segment with the given sequence number.
pub fn segment_path(dir: &Path, seq: u64) -> PathBuf {
    dir.join(format!("spool_{seq:06}.dat"))
}

/// List the sequence numbers of all existing segment files in `dir`.
async fn list_segments(dir: &Path) -> Result<Vec<u64>, SinkError> {
    let mut seqs = Vec::new();
    let mut rd = fs::read_dir(dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("spool_") && name_str.ends_with(".dat") {
            let seq_part = &name_str[6..name_str.len() - 4]; // strip "spool_" and ".dat"
            if let Ok(seq) = seq_part.parse::<u64>() {
                seqs.push(seq);
            }
        }
    }
    Ok(seqs)
}

/// Open a segment file for appending (creating if needed).
async fn open_segment_for_append(path: &Path) -> Result<File, SinkError> {
    Ok(OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?)
}

/// Scan `path` for complete records; truncate at the first incomplete record.
/// Returns an open file handle positioned at EOF and the byte count of valid data.
async fn recover_segment(path: &Path) -> Result<(File, u64), SinkError> {
    // First pass: read-only scan to find last good offset.
    let mut scan = File::open(path).await?;
    let mut good_offset: u64 = 0;

    loop {
        // Try to read a 4-byte length header.
        let mut hdr = [0u8; 4];
        match scan.read_exact(&mut hdr).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(hdr) as u64;

        // Try to skip `len` bytes of JSON body.
        let pos_before_skip = scan.stream_position().await?;
        let mut remaining = len;
        let mut buf = vec![0u8; remaining.min(4096) as usize];
        while remaining > 0 {
            let to_read = remaining.min(buf.len() as u64) as usize;
            match scan.read_exact(&mut buf[..to_read]).await {
                Ok(_) => remaining -= to_read as u64,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Body is incomplete; good_offset stands.
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        if remaining == 0 {
            // Full record read successfully.
            good_offset += 4 + len;
        } else {
            // Incomplete body — stop scanning.
            break;
        }

        let _ = pos_before_skip; // suppress unused warning
    }
    drop(scan);

    // If the file has extra bytes beyond good_offset, truncate it.
    let file_meta = tokio::fs::metadata(path).await?;
    if file_meta.len() != good_offset {
        let path_clone = path.to_path_buf();
        tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            let f = std::fs::OpenOptions::new().write(true).open(path_clone)?;
            f.set_len(good_offset)
        })
        .await
        .map_err(|e| SinkError::SpoolIo(std::io::Error::other(e.to_string())))??;
    }

    // Open for append at the clean position.
    let mut file = OpenOptions::new().append(true).open(path).await?;
    // seek is implicit with append mode; verify position is at good_offset.
    let pos = file.seek(std::io::SeekFrom::End(0)).await?;
    debug_assert_eq!(pos, good_offset);

    Ok((file, good_offset))
}

/// Read all complete records from a segment file.
pub async fn read_all_records(path: &Path) -> Result<Vec<Envelope>, SinkError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut file = File::open(path).await?;
    let mut envelopes = Vec::new();

    loop {
        let mut hdr = [0u8; 4];
        match file.read_exact(&mut hdr).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(hdr) as usize;
        let len = u32::from_le_bytes(hdr) as usize;
        // レコード長が大きすぎる場合はエラーを返す
        if len > 10 * 1024 * 1024 { // 例: 10MBの制限
            return Err(SinkError::Other(format!("レコードが大きすぎます: {} バイト", len)));
        }
        let mut body = vec![0u8; len];
        match file.read_exact(&mut body).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        if let Ok(env) = serde_json::from_slice::<Envelope>(&body) {
            envelopes.push(env);
        }
    }
    Ok(envelopes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write as _;
    use tempfile::TempDir;

    fn make_envelope(channel: &str) -> Envelope {
        Envelope {
            message_id: None,
            sequence: None,
            exchange: "test".to_string(),
            channel: channel.to_string(),
            symbol: "BTC/USDT".to_string(),
            server_time_ms: None,
            received_at_ms: 0,
            payload: json!({"x": 1}),
        }
    }

    async fn make_spool(dir: &Path, max_segment_bytes: u64, max_total_bytes: u64) -> Arc<DurableSpool> {
        let config = SpoolConfig {
            dir: dir.to_path_buf(),
            max_segment_bytes,
            max_total_bytes,
            on_full: OnFullPolicy::DropAll,
        };
        let metrics = PersistenceMetrics::new();
        DurableSpool::open(config, metrics).await.unwrap()
    }

    #[tokio::test]
    async fn write_and_read_back() {
        let tmp = TempDir::new().unwrap();
        let spool = make_spool(tmp.path(), 1024 * 1024, 10 * 1024 * 1024).await;

        let envs = vec![
            make_envelope("trades"),
            make_envelope("orderbook"),
        ];
        let n = spool.append_batch(envs.clone()).await.unwrap();
        assert_eq!(n, 2);

        // There's one write segment (current); no complete segments yet.
        let complete = spool.complete_segments().await.unwrap();
        assert!(complete.is_empty(), "expected no complete segs yet, got: {complete:?}");

        // Read the current (only) segment directly via segment_path.
        let current_seq = spool.current_seq.load(std::sync::atomic::Ordering::Acquire);
        let path = segment_path(tmp.path(), current_seq);
        let read_back = read_all_records(&path).await.unwrap();
        assert_eq!(read_back.len(), 2);
        assert_eq!(read_back[0].channel, "trades");
        assert_eq!(read_back[1].channel, "orderbook");
    }

    #[tokio::test]
    async fn segment_rotation_by_size() {
        let tmp = TempDir::new().unwrap();
        // Max segment = 80 bytes; a single envelope JSON is ~120 bytes.
        // Every write should rotate.
        let spool = make_spool(tmp.path(), 80, 10 * 1024 * 1024).await;

        for i in 0..3u32 {
            spool
                .append_batch(vec![Envelope {
                    message_id: Some(format!("id-{i}")),
                    sequence: Some(i as u64),
                    exchange: "test".to_string(),
                    channel: "trades".to_string(),
                    symbol: "BTC/USDT".to_string(),
                    server_time_ms: None,
                    received_at_ms: 0,
                    payload: json!({"i": i}),
                }])
                .await
                .unwrap();
        }

        let segs = list_segments(tmp.path()).await.unwrap();
        // Each envelope triggered a rotation → at least 3 segments.
        assert!(segs.len() >= 3, "expected ≥3 segments, got {}", segs.len());
    }

    #[tokio::test]
    async fn on_full_drop_all_drops_silently() {
        let tmp = TempDir::new().unwrap();
        // Cap is 0 bytes effectively (50 bytes) — everything gets dropped.
        let spool = make_spool(tmp.path(), 1024 * 1024, 50).await;

        let batch = vec![make_envelope("trades"), make_envelope("orderbook")];
        let n = spool.append_batch(batch).await.unwrap();
        assert_eq!(n, 0, "expected 0 written (all dropped), got {n}");
        assert_eq!(spool.metrics.spool_dropped_total.total(), 2);
    }

    #[tokio::test]
    async fn on_full_drop_ticker_depth_keeps_trades() {
        let tmp = TempDir::new().unwrap();
        let config = SpoolConfig {
            dir: tmp.path().to_path_buf(),
            max_segment_bytes: 1024 * 1024,
            max_total_bytes: 50, // force full immediately
            on_full: OnFullPolicy::DropTickerDepthKeepTrade,
        };
        let metrics = PersistenceMetrics::new();
        let spool = DurableSpool::open(config, metrics).await.unwrap();

        // orderbook channel → dropped
        let n = spool
            .append_batch(vec![make_envelope("orderbook")])
            .await
            .unwrap();
        assert_eq!(n, 0);
        assert_eq!(spool.metrics.spool_dropped_total.get("test", "orderbook"), 1);
    }

    #[tokio::test]
    async fn partial_write_recovery() {
        let tmp = TempDir::new().unwrap();
        let seg_path = segment_path(tmp.path(), 1);

        // Write one complete record manually.
        let env = make_envelope("trades");
        let json = serde_json::to_vec(&env).unwrap();
        let len = json.len() as u32;
        {
            let mut f = std::fs::File::create(&seg_path).unwrap();
            f.write_all(&len.to_le_bytes()).unwrap();
            f.write_all(&json).unwrap();

            // Write a partial record: header only, no body.
            f.write_all(&999u32.to_le_bytes()).unwrap();
            // (no body bytes follow — simulates crash mid-write)
        }

        // Open spool; recovery should truncate the partial record.
        let (_, good_bytes) = recover_segment(&seg_path).await.unwrap();
        assert_eq!(good_bytes, (4 + json.len()) as u64);

        let records = read_all_records(&seg_path).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].channel, "trades");
    }

    #[tokio::test]
    async fn delete_segment_updates_metrics() {
        let tmp = TempDir::new().unwrap();
        let spool = make_spool(tmp.path(), 1024 * 1024, 10 * 1024 * 1024).await;

        // Write enough envelopes to create a second (complete) segment.
        // We do this by writing to a small-segment spool.
        let config = SpoolConfig {
            dir: tmp.path().join("sub"),
            max_segment_bytes: 10, // tiny → immediate rotation
            max_total_bytes: 10 * 1024 * 1024,
            on_full: OnFullPolicy::DropAll,
        };
        let metrics = PersistenceMetrics::new();
        let spool2 = DurableSpool::open(config, metrics.clone()).await.unwrap();

        spool2.append_batch(vec![make_envelope("trades")]).await.unwrap();
        spool2.append_batch(vec![make_envelope("trades")]).await.unwrap();

        let complete = spool2.complete_segments().await.unwrap();
        assert!(!complete.is_empty(), "expected at least one complete segment");

        let bytes_before = metrics.spool_bytes();
        spool2.delete_segment(complete[0]).await.unwrap();
        let bytes_after = metrics.spool_bytes();
        assert!(bytes_after < bytes_before, "spool_bytes should decrease after delete");

        let _ = spool; // suppress unused
    }
}
