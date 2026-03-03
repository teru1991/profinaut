use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct MonotonicCounter {
    v: AtomicU64,
}

impl Default for MonotonicCounter {
    fn default() -> Self {
        Self {
            v: AtomicU64::new(1),
        }
    }
}

impl MonotonicCounter {
    pub fn next(&self) -> u64 {
        self.v.fetch_add(1, Ordering::Relaxed)
    }
}

pub fn make_client_order_id(prefix: &str, run_id: &str, seq: u64) -> String {
    let p = sanitize(prefix);
    let r = sanitize(run_id);
    format!("{p}_{r}_{seq}")
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
            _ => '_',
        })
        .collect()
}
