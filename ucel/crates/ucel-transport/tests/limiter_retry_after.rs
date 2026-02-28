use std::time::{Duration, Instant};

use ucel_core::{ErrorCode, UcelError};

use ucel_transport::http::limiter::{
    BucketConfig, HttpRateLimiter, HttpRateLimiterConfig, VenueLimiterConfig,
};
use ucel_transport::http::retry::{decide_retry, parse_retry_after_ms, RetryDecision};
use ucel_transport::RetryPolicy;

#[test]
fn retry_after_parse_is_stable_and_tolerant() {
    // seconds (no unit, small value) => seconds
    assert_eq!(parse_retry_after_ms("2"), Some(2000));
    assert_eq!(parse_retry_after_ms(" 2 "), Some(2000));
    assert_eq!(parse_retry_after_ms("2s"), Some(2000));

    // ms explicitly
    assert_eq!(parse_retry_after_ms("1500ms"), Some(1500));

    // heuristic: large number with no unit => ms
    assert_eq!(parse_retry_after_ms("1500"), Some(1500));

    // edge cases
    assert_eq!(parse_retry_after_ms(""), None);
    assert_eq!(parse_retry_after_ms("   "), None);
    assert_eq!(parse_retry_after_ms("abc"), None);
}

#[test]
fn retry_decision_respects_retry_after_when_enabled() {
    let policy = RetryPolicy {
        base_delay_ms: 100,
        max_delay_ms: 10_000,
        jitter_ms: 0,
        respect_retry_after: true,
    };

    let err = UcelError::new(ErrorCode::RateLimited, "rl").with_retry_after_ms(777);

    match decide_retry(&policy, 0, &err) {
        RetryDecision::RetryAfter(d) => assert_eq!(d, Duration::from_millis(777)),
        _ => panic!("expected RetryAfter"),
    }
}

#[test]
fn retry_decision_uses_backoff_when_retry_after_missing() {
    let policy = RetryPolicy {
        base_delay_ms: 100,
        max_delay_ms: 10_000,
        jitter_ms: 0,
        respect_retry_after: true,
    };

    // retryable error without retry-after -> backoff
    let err = UcelError::new(ErrorCode::Timeout, "t");
    match decide_retry(&policy, 2, &err) {
        RetryDecision::RetryAfter(d) => {
            // attempt=2 => 100 * 2^2 = 400ms
            assert_eq!(d, Duration::from_millis(400));
        }
        _ => panic!("expected RetryAfter"),
    }
}

#[test]
fn retry_decision_does_not_retry_non_retryable() {
    let policy = RetryPolicy {
        base_delay_ms: 100,
        max_delay_ms: 10_000,
        jitter_ms: 0,
        respect_retry_after: true,
    };

    let err = UcelError::new(ErrorCode::InvalidOrder, "bad");
    assert_eq!(decide_retry(&policy, 0, &err), RetryDecision::DoNotRetry);
}

#[tokio::test(flavor = "current_thread")]
async fn http_rate_limiter_is_bucketed_by_venue_and_auth() {
    // Make public very tight and private separate.
    let mut cfg = HttpRateLimiterConfig::default();
    cfg.default = VenueLimiterConfig {
        public: BucketConfig {
            capacity: 1.0,
            refill_per_sec: 1.0, // 1 rps
        },
        private: BucketConfig {
            capacity: 1.0,
            refill_per_sec: 10.0, // 10 rps (effectively no wait)
        },
    };

    let lim = HttpRateLimiter::new(cfg);

    let t0 = Instant::now();

    // First public token: no wait
    let w0 = lim.acquire_wait("venueA", false, t0).await;
    assert_eq!(w0, Duration::from_secs(0));

    // Second public token at same time: must wait (public bucket empty)
    let w1 = lim.acquire_wait("venueA", false, t0).await;
    assert!(w1 > Duration::from_millis(0));

    // Private bucket should be separate: no wait even while public is empty
    let w_priv = lim.acquire_wait("venueA", true, t0).await;
    assert_eq!(w_priv, Duration::from_secs(0));

    // Different venue: separate buckets (public should again be no wait)
    let w_other = lim.acquire_wait("venueB", false, t0).await;
    assert_eq!(w_other, Duration::from_secs(0));
}
