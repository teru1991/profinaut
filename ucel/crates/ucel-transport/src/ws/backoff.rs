use crate::ws::reconnect::backoff_with_jitter_ms;

pub fn ingest_backoff_ms(attempt: u32, base: u64, max: u64, jitter: u64) -> u64 {
    backoff_with_jitter_ms(attempt, base, max, jitter)
}
