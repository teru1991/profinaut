use serde_json::Value;
use ucel_core::{ErrorCode, UcelError};
use ucel_transport::ws::adapter::InboundJsonGuard;

#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_usize(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            0
        } else {
            (self.next_u64() as usize) % upper_exclusive
        }
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }
}

pub fn mutate_bytes(input: &[u8], rng: &mut XorShift64, max_len: usize) -> Vec<u8> {
    let mut out = input.to_vec();
    match rng.next_usize(5) {
        0 => {
            if !out.is_empty() {
                let idx = rng.next_usize(out.len());
                out[idx] ^= 1 << (rng.next_usize(8) as u8);
            }
        }
        1 => {
            if out.len() < max_len {
                let idx = if out.is_empty() {
                    0
                } else {
                    rng.next_usize(out.len() + 1)
                };
                out.insert(idx, rng.next_u8());
            }
        }
        2 => {
            if !out.is_empty() {
                let idx = rng.next_usize(out.len());
                out.remove(idx);
            }
        }
        3 => {
            if !out.is_empty() && out.len() < max_len {
                let start = rng.next_usize(out.len());
                let remaining = out.len() - start;
                let take = 1 + rng.next_usize(remaining.min(max_len - out.len()).max(1));
                let chunk: Vec<u8> = out[start..start + take.min(remaining)].to_vec();
                out.extend_from_slice(&chunk);
            }
        }
        _ => {
            if !out.is_empty() {
                let start = rng.next_usize(out.len());
                let end = start + 1 + rng.next_usize(out.len() - start);
                out = out[start..end].to_vec();
            }
        }
    }

    if out.len() > max_len {
        out.truncate(max_len);
    }
    out
}

pub fn json_depth(value: &Value, max_depth: usize) -> Result<usize, UcelError> {
    fn walk(
        v: &Value,
        depth: usize,
        max_depth: usize,
        max_seen: &mut usize,
    ) -> Result<(), UcelError> {
        *max_seen = (*max_seen).max(depth);
        if depth > max_depth {
            return Err(UcelError::new(
                ErrorCode::WsProtocolViolation,
                format!("json depth exceeded: {depth} (max {max_depth})"),
            ));
        }

        match v {
            Value::Array(items) => {
                for item in items {
                    walk(item, depth + 1, max_depth, max_seen)?;
                }
            }
            Value::Object(map) => {
                for item in map.values() {
                    walk(item, depth + 1, max_depth, max_seen)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    let mut max_seen = 0;
    walk(value, 0, max_depth, &mut max_seen)?;
    Ok(max_seen)
}

pub fn ws_guard_entry(raw: &[u8]) -> Result<(), UcelError> {
    InboundJsonGuard::default().enforce(raw)
}
