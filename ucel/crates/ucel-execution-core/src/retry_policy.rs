#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryClass {
    Retryable,
    NonRetryable,
    Cooldown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetryDecision {
    pub class: RetryClass,
    pub max_attempts: u32,
    pub cooldown_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    ReadOnly,
    PlaceOrder,
    CancelOrder,
    AmendOrder,
}

pub fn decide(
    op: OpKind,
    http_status: Option<u16>,
    is_timeout: bool,
    has_idempotency: bool,
) -> RetryDecision {
    if is_timeout {
        return for_timeout(op, has_idempotency);
    }
    match http_status {
        Some(429) => RetryDecision {
            class: RetryClass::Cooldown,
            max_attempts: 5,
            cooldown_ms: 1_500,
        },
        Some(500..=599) => for_timeout(op, has_idempotency),
        Some(401 | 403) | Some(400..=499) => RetryDecision {
            class: RetryClass::NonRetryable,
            max_attempts: 0,
            cooldown_ms: 0,
        },
        _ => RetryDecision {
            class: RetryClass::NonRetryable,
            max_attempts: 0,
            cooldown_ms: 0,
        },
    }
}

fn for_timeout(op: OpKind, has_idempotency: bool) -> RetryDecision {
    match op {
        OpKind::ReadOnly => RetryDecision {
            class: RetryClass::Retryable,
            max_attempts: 3,
            cooldown_ms: 0,
        },
        _ if has_idempotency => RetryDecision {
            class: RetryClass::Retryable,
            max_attempts: 2,
            cooldown_ms: 0,
        },
        _ => RetryDecision {
            class: RetryClass::NonRetryable,
            max_attempts: 0,
            cooldown_ms: 0,
        },
    }
}
