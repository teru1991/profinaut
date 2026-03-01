use std::time::Duration;

use ucel_transport::ws::connection::{apply_stability_overrides_for_test, WsRunConfig};
use ucel_ws_rules::ExchangeWsRules;

#[test]
fn rules_stability_overrides_are_applied() {
    let rules: ExchangeWsRules = toml::from_str(
        r#"
exchange_id = "x"
support_level = "full"

[rate]
messages_per_second = 10
messages_per_hour = 3600

[stability.circuit_breaker]
failure_threshold = 7
success_threshold = 2
cooldown_ms = 1234
half_open_max_trials = 4

[stability.rate_limit]
max_attempts = 9
base_cooldown_secs = 2
max_cooldown_secs = 30
default_penalty_ms = 777

[stability.stale]
stale_after_secs = 33
sweep_interval_ms = 1500
max_batch = 22

[stability.graceful]
drain_timeout_ms = 6000
join_timeout_ms = 7000
"#,
    )
    .unwrap();

    let cfg = apply_stability_overrides_for_test(WsRunConfig::default(), &rules);

    assert_eq!(cfg.breaker.failure_threshold, 7);
    assert_eq!(cfg.breaker.success_threshold, 2);
    assert_eq!(cfg.breaker.cooldown, Duration::from_millis(1234));
    assert_eq!(cfg.breaker.half_open_max_trials, 4);

    assert_eq!(cfg.rl_max_attempts, 9);
    assert_eq!(cfg.rl_base_cooldown_secs, 2);
    assert_eq!(cfg.rl_max_cooldown_secs, 30);

    assert_eq!(cfg.stale_after, Duration::from_secs(33));
    assert_eq!(cfg.stale_sweep_interval, Duration::from_millis(1500));
    assert_eq!(cfg.stale_max_batch, 22);

    assert_eq!(cfg.graceful.drain_timeout, Duration::from_millis(6000));
    assert_eq!(cfg.graceful.join_timeout, Duration::from_millis(7000));
}
