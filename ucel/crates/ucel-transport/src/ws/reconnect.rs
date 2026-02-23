use rand::Rng;

/// 指数バックオフ + ジッタ（ms）
pub fn backoff_with_jitter_ms(attempt: u32, base_ms: u64, max_ms: u64, jitter_ms: u64) -> u64 {
    let exp = 2u64.saturating_pow(attempt.min(16));
    let mut ms = base_ms.saturating_mul(exp);
    if ms > max_ms {
        ms = max_ms;
    }
    let mut rng = rand::thread_rng();
    let j = rng.gen_range(0..=jitter_ms);
    ms.saturating_add(j)
}

/// reconnect storm guard
/// - window 内の reconnect 回数が max を超えたら false
/// - ここでは「カウントがmax以下ならOK」という単純ガード（window管理は呼び出し側が実施）
pub fn storm_guard(reconnect_count_in_window: usize, max: usize) -> bool {
    reconnect_count_in_window <= max
}
