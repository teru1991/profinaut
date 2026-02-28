//! Priority outbound queue for WS.
//!
//! Requirements:
//! - Private priority (auth/ordering) over public streams.
//! - Deterministic overflow handling (drop/slowdown/spill).
//! - A shutdown close marker that *drains* before closing the socket.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, Notify};

use super::overflow::{DropMode, OverflowPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundPriority {
    /// Highest priority, control-plane frames.
    Control,
    /// Private/auth streams.
    Private,
    /// Public streams.
    Public,
}

impl OutboundPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboundPriority::Control => "control",
            OutboundPriority::Private => "private",
            OutboundPriority::Public => "public",
        }
    }
}

/// Heuristic op_id classifier.
///
/// Rule (fixed): treat any op_id containing `.private.` or starting with `crypto.private.` as private.
/// Everything else is public.
pub fn classify_op_id_priority(op_id: &str) -> OutboundPriority {
    let s = op_id.to_ascii_lowercase();
    if s.contains(".private.") || s.starts_with("crypto.private.") {
        OutboundPriority::Private
    } else {
        OutboundPriority::Public
    }
}

#[derive(Debug, Clone)]
pub enum WsOutboundFrame {
    Text(String),
    Pong(Vec<u8>),
    /// A close request marker.
    ///
    /// The writer should drain pending frames then send a WS close.
    CloseRequest,
}

impl WsOutboundFrame {
    pub fn kind(&self) -> &'static str {
        match self {
            WsOutboundFrame::Text(_) => "text",
            WsOutboundFrame::Pong(_) => "pong",
            WsOutboundFrame::CloseRequest => "close_request",
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            WsOutboundFrame::Text(s) => s.as_bytes().to_vec(),
            WsOutboundFrame::Pong(p) => p.clone(),
            WsOutboundFrame::CloseRequest => b"close_request".to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueuedOutbound {
    pub priority: OutboundPriority,
    pub op_id: Option<String>,
    pub symbol: Option<String>,
    pub frame: WsOutboundFrame,
    pub meta: serde_json::Value,
}

impl QueuedOutbound {
    pub fn close_request() -> Self {
        Self {
            priority: OutboundPriority::Control,
            op_id: None,
            symbol: None,
            frame: WsOutboundFrame::CloseRequest,
            meta: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    Enqueued,
    Dropped,
    Spilled,
}

#[derive(Debug)]
struct Inner {
    control: VecDeque<QueuedOutbound>,
    private: VecDeque<QueuedOutbound>,
    public: VecDeque<QueuedOutbound>,
}

impl Inner {
    fn new() -> Self {
        Self {
            control: VecDeque::new(),
            private: VecDeque::new(),
            public: VecDeque::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.control.is_empty() && self.private.is_empty() && self.public.is_empty()
    }

    fn push(&mut self, item: QueuedOutbound) {
        match item.priority {
            OutboundPriority::Control => self.control.push_back(item),
            OutboundPriority::Private => self.private.push_back(item),
            OutboundPriority::Public => self.public.push_back(item),
        }
    }

    fn pop(&mut self) -> Option<QueuedOutbound> {
        // Control first, then private, then public.
        if let Some(x) = self.control.pop_front() {
            return Some(x);
        }
        if let Some(x) = self.private.pop_front() {
            return Some(x);
        }
        self.public.pop_front()
    }

    fn drop_oldest_low_priority(&mut self) -> bool {
        // Drop from public first, then private. Control is never dropped.
        if self.public.pop_front().is_some() {
            return true;
        }
        if self.private.pop_front().is_some() {
            return true;
        }
        false
    }
}

/// An async priority queue with bounded capacity.
#[derive(Debug)]
pub struct PriorityQueue {
    cap: usize,
    len: AtomicUsize,
    inner: Mutex<Inner>,
    item_notify: Notify,
    space_notify: Notify,
    closed: AtomicBool,
    closing: AtomicBool,
}

impl PriorityQueue {
    pub fn new(cap: usize) -> Arc<Self> {
        Arc::new(Self {
            cap: cap.max(1),
            len: AtomicUsize::new(0),
            inner: Mutex::new(Inner::new()),
            item_notify: Notify::new(),
            space_notify: Notify::new(),
            closed: AtomicBool::new(false),
            closing: AtomicBool::new(false),
        })
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    pub async fn is_empty(&self) -> bool {
        let g = self.inner.lock().await;
        g.is_empty()
    }

    /// Begin shutdown: reject new non-control frames.
    pub fn begin_closing(&self) {
        self.closing.store(true, Ordering::SeqCst);
    }

    /// Close queue: receiver returns None after draining.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.item_notify.notify_waiters();
        self.space_notify.notify_waiters();
    }

    fn can_accept(&self, item: &QueuedOutbound) -> bool {
        if self.closed.load(Ordering::SeqCst) {
            return false;
        }
        if self.closing.load(Ordering::SeqCst) {
            // During closing, only allow control frames.
            return item.priority == OutboundPriority::Control;
        }
        true
    }

    /// Push with overflow policy.
    pub async fn push(
        &self,
        exchange_id: &str,
        conn_id: &str,
        item: QueuedOutbound,
        policy: &OverflowPolicy,
        spool_ts_unix: u64,
    ) -> Result<PushOutcome, String> {
        if !self.can_accept(&item) {
            return Ok(PushOutcome::Dropped);
        }

        // Fast path.
        if self.len() < self.cap {
            let mut g = self.inner.lock().await;
            if self.len() < self.cap {
                g.push(item);
                self.len.fetch_add(1, Ordering::SeqCst);
                drop(g);
                self.item_notify.notify_one();
                return Ok(PushOutcome::Enqueued);
            }
        }

        // Slow path: overflow.
        match policy {
            OverflowPolicy::Drop { mode } => Ok(self.apply_drop_mode(item, *mode).await),

            OverflowPolicy::SlowDown { max_wait, fallback } => {
                let start = Instant::now();
                while start.elapsed() < *max_wait {
                    let remaining = max_wait.saturating_sub(start.elapsed());
                    let _ = tokio::time::timeout(
                        remaining.min(Duration::from_millis(50)),
                        self.space_notify.notified(),
                    )
                    .await;

                    if self.closed.load(Ordering::SeqCst) {
                        return Ok(PushOutcome::Dropped);
                    }
                    if self.len() < self.cap {
                        let mut g = self.inner.lock().await;
                        if self.len() < self.cap {
                            g.push(item);
                            self.len.fetch_add(1, Ordering::SeqCst);
                            drop(g);
                            self.item_notify.notify_one();
                            return Ok(PushOutcome::Enqueued);
                        }
                    }
                }

                Ok(self.apply_drop_mode(item, *fallback).await)
            }

            OverflowPolicy::SpillToDisk { spooler, fallback } => {
                let bytes = item.frame.to_bytes();
                let op = item.op_id.clone().unwrap_or_else(|| "unknown".to_string());
                let sym = item.symbol.clone();

                let spilled = spooler
                    .spill_bytes(
                        exchange_id,
                        conn_id,
                        &op,
                        sym.as_deref(),
                        item.frame.kind(),
                        item.priority.as_str(),
                        &bytes,
                        item.meta.clone(),
                        spool_ts_unix,
                    )
                    .await;

                match spilled {
                    Ok(()) => Ok(PushOutcome::Spilled),
                    Err(_) => Ok(self.apply_drop_mode(item, *fallback).await),
                }
            }
        }
    }

    async fn apply_drop_mode(&self, item: QueuedOutbound, mode: DropMode) -> PushOutcome {
        match mode {
            DropMode::DropNewest => PushOutcome::Dropped,
            DropMode::DropOldestLowPriority => {
                let mut g = self.inner.lock().await;
                let dropped = g.drop_oldest_low_priority();
                if dropped {
                    g.push(item);
                    drop(g);
                    self.item_notify.notify_one();
                    PushOutcome::Enqueued
                } else {
                    PushOutcome::Dropped
                }
            }
        }
    }

    /// Receive next item (priority order). Returns None only when closed and empty.
    pub async fn recv(&self) -> Option<QueuedOutbound> {
        loop {
            {
                let mut g = self.inner.lock().await;
                if let Some(x) = g.pop() {
                    self.len.fetch_sub(1, Ordering::SeqCst);
                    drop(g);
                    self.space_notify.notify_one();
                    return Some(x);
                }
                if self.closed.load(Ordering::SeqCst) {
                    return None;
                }
            }

            self.item_notify.notified().await;
        }
    }
}

pub fn outcome_is_fatal(_o: PushOutcome) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::overflow::{OverflowPolicy, Spooler, SpoolerConfig};

    #[tokio::test(flavor = "current_thread")]
    async fn priority_order_control_private_public() {
        let q = PriorityQueue::new(10);

        let policy = OverflowPolicy::drop_newest();
        let ts = 1u64;

        q.push(
            "x",
            "c",
            QueuedOutbound {
                priority: OutboundPriority::Public,
                op_id: Some("p".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("p".into()),
                meta: serde_json::json!({}),
            },
            &policy,
            ts,
        )
        .await
        .unwrap();

        q.push(
            "x",
            "c",
            QueuedOutbound {
                priority: OutboundPriority::Private,
                op_id: Some("priv".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("priv".into()),
                meta: serde_json::json!({}),
            },
            &policy,
            ts,
        )
        .await
        .unwrap();

        q.push(
            "x",
            "c",
            QueuedOutbound {
                priority: OutboundPriority::Control,
                op_id: Some("ctl".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("ctl".into()),
                meta: serde_json::json!({}),
            },
            &policy,
            ts,
        )
        .await
        .unwrap();

        assert_eq!(q.recv().await.unwrap().frame.to_bytes(), b"ctl");
        assert_eq!(q.recv().await.unwrap().frame.to_bytes(), b"priv");
        assert_eq!(q.recv().await.unwrap().frame.to_bytes(), b"p");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn overflow_drop_oldest_low_priority() {
        let q = PriorityQueue::new(1);
        let policy = OverflowPolicy::Drop {
            mode: DropMode::DropOldestLowPriority,
        };

        let ts = 1u64;
        q.push(
            "x",
            "c",
            QueuedOutbound {
                priority: OutboundPriority::Public,
                op_id: Some("a".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("a".into()),
                meta: serde_json::json!({}),
            },
            &OverflowPolicy::drop_newest(),
            ts,
        )
        .await
        .unwrap();

        let out = q
            .push(
                "x",
                "c",
                QueuedOutbound {
                    priority: OutboundPriority::Private,
                    op_id: Some("b".into()),
                    symbol: None,
                    frame: WsOutboundFrame::Text("b".into()),
                    meta: serde_json::json!({}),
                },
                &policy,
                ts,
            )
            .await
            .unwrap();

        assert_eq!(out, PushOutcome::Enqueued);
        assert_eq!(q.recv().await.unwrap().frame.to_bytes(), b"b");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn overflow_spill_to_disk_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let sp = Arc::new(Spooler::open(SpoolerConfig::new(dir.path())).unwrap());
        let q = PriorityQueue::new(1);

        let ts = 1u64;
        q.push(
            "x",
            "c",
            QueuedOutbound {
                priority: OutboundPriority::Public,
                op_id: Some("a".into()),
                symbol: None,
                frame: WsOutboundFrame::Text("a".into()),
                meta: serde_json::json!({}),
            },
            &OverflowPolicy::drop_newest(),
            ts,
        )
        .await
        .unwrap();

        let policy = OverflowPolicy::SpillToDisk {
            spooler: sp,
            fallback: DropMode::DropNewest,
        };

        let out = q
            .push(
                "x",
                "c",
                QueuedOutbound {
                    priority: OutboundPriority::Public,
                    op_id: Some("b".into()),
                    symbol: None,
                    frame: WsOutboundFrame::Text("b".into()),
                    meta: serde_json::json!({"n":1}),
                },
                &policy,
                ts,
            )
            .await
            .unwrap();

        assert_eq!(out, PushOutcome::Spilled);
        let files: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert!(!files.is_empty());
    }
}
