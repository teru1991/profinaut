use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize)]
pub struct StabilityEvent {
    pub ts_unix: u64,
    pub exchange_id: String,
    pub conn_id: String,
    pub kind: String,
    pub fields: serde_json::Value,
}

impl StabilityEvent {
    pub fn now(exchange_id: &str, conn_id: &str, kind: &str, fields: serde_json::Value) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            ts_unix: ts,
            exchange_id: exchange_id.to_string(),
            conn_id: conn_id.to_string(),
            kind: kind.to_string(),
            fields,
        }
    }
}

#[derive(Debug)]
pub struct StabilityEventRing {
    cap: usize,
    inner: Mutex<VecDeque<StabilityEvent>>,
}

impl StabilityEventRing {
    pub fn new(cap: usize) -> Arc<Self> {
        Arc::new(Self {
            cap: cap.max(64),
            inner: Mutex::new(VecDeque::new()),
        })
    }

    pub fn push(&self, ev: StabilityEvent) {
        let mut g = self.inner.lock().unwrap();
        if g.len() >= self.cap {
            g.pop_front();
        }
        g.push_back(ev);
    }

    pub fn snapshot(&self) -> Vec<StabilityEvent> {
        let g = self.inner.lock().unwrap();
        g.iter().cloned().collect()
    }
}
