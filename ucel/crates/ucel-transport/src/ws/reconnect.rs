use rand::Rng;

pub fn backoff_with_jitter_ms(attempt: u32, base_ms: u64, max_ms: u64, jitter_ms: u64) -> u64 {
    let exp = base_ms.saturating_mul(2u64.saturating_pow(attempt));
    let capped = exp.min(max_ms);
    let j = if jitter_ms == 0 { 0 } else { rand::thread_rng().gen_range(0..=jitter_ms) };
    capped.saturating_add(j).min(max_ms.saturating_add(jitter_ms))
}

pub fn storm_guard(all_reconnects_within_window: usize, max_allowed: usize) -> bool {
    all_reconnects_within_window <= max_allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_increases() {
        let d0 = backoff_with_jitter_ms(0, 100, 10_000, 0);
        let d1 = backoff_with_jitter_ms(1, 100, 10_000, 0);
        assert!(d1 >= d0);
    }
}
