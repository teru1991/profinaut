//! HTTP retry utilities.
//!
//! This module intentionally does not depend on any specific HTTP client.
//! It relies on `UcelError.retry_after_ms` when available.

use std::time::Duration;

use ucel_core::{ErrorCode, UcelError};

use crate::{classify_error, next_retry_delay_ms, RetryClass, RetryPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    DoNotRetry,
    RetryAfter(Duration),
}

/// Parse a Retry-After value into milliseconds.
///
/// Supports common exchange variants:
/// - "2" (seconds)
/// - "2s" (seconds)
/// - "1500" (milliseconds, if large)
/// - "1500ms" (milliseconds)
///
/// Heuristic:
/// - If no unit is present and value <= 60 => seconds
/// - Otherwise => milliseconds
pub fn parse_retry_after_ms(value: &str) -> Option<u64> {
    let v = value.trim().to_ascii_lowercase();
    if v.is_empty() {
        return None;
    }
    if let Some(num) = v.strip_suffix("ms") {
        return num.trim().parse::<u64>().ok();
    }
    if let Some(num) = v.strip_suffix('s') {
        let s = num.trim().parse::<f64>().ok()?;
        return Some((s * 1000.0).round().max(0.0) as u64);
    }
    let n = v.parse::<u64>().ok()?;
    if n <= 60 {
        Some(n.saturating_mul(1000))
    } else {
        Some(n)
    }
}

/// Decide retry behavior from an error and policy.
///
/// - Respects `retry_after_ms` if the policy enables it.
/// - Applies exponential backoff + jitter otherwise.
pub fn decide_retry(policy: &RetryPolicy, attempt: u32, err: &UcelError) -> RetryDecision {
    if classify_error(&err.code) == RetryClass::NonRetryable {
        return RetryDecision::DoNotRetry;
    }

    let ra = err.retry_after_ms;
    let ms = next_retry_delay_ms(policy, attempt, ra);
    RetryDecision::RetryAfter(Duration::from_millis(ms))
}

/// Helper to create a `RateLimited` error with Retry-After.
pub fn rate_limited(retry_after_ms: u64, message: impl Into<String>) -> UcelError {
    UcelError::new(ErrorCode::RateLimited, message).with_retry_after_ms(retry_after_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decides_do_not_retry_for_non_retryable() {
        let p = RetryPolicy {
            base_delay_ms: 50,
            max_delay_ms: 1000,
            jitter_ms: 0,
            respect_retry_after: true,
        };
        let err = UcelError::new(ErrorCode::InvalidOrder, "no");
        assert_eq!(decide_retry(&p, 0, &err), RetryDecision::DoNotRetry);
    }

    #[test]
    fn respects_retry_after_when_present() {
        let p = RetryPolicy {
            base_delay_ms: 50,
            max_delay_ms: 1000,
            jitter_ms: 0,
            respect_retry_after: true,
        };
        let err = rate_limited(333, "rl");
        match decide_retry(&p, 0, &err) {
            RetryDecision::RetryAfter(d) => assert_eq!(d, Duration::from_millis(333)),
            _ => panic!("expected retry"),
        }
    }

    #[test]
    fn parses_retry_after_seconds_and_ms() {
        assert_eq!(parse_retry_after_ms("2"), Some(2000));
        assert_eq!(parse_retry_after_ms("2s"), Some(2000));
        assert_eq!(parse_retry_after_ms("1500"), Some(1500));
        assert_eq!(parse_retry_after_ms("1500ms"), Some(1500));
        assert_eq!(parse_retry_after_ms(""), None);
    }
}
