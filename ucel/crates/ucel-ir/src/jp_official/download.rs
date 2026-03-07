use super::access::JpPolitenessPolicy;

pub fn retry_backoff_ms(attempt: u8, policy: JpPolitenessPolicy) -> u64 {
    policy.base_backoff_ms.saturating_mul((attempt as u64) + 1)
}
