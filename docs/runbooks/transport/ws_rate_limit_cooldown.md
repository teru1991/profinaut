# Runbook: WS Rate-limit Cooldown

1. Identify Nack reasons containing rate-limit keywords or 429.
2. Confirm limiter penalty was applied and cooldown timestamp is set.
3. Verify pending batch skips cooldown rows until expiry.
4. If attempts exceed threshold, confirm deadletter transition.
5. Tune `rl_base_cooldown_secs`, `rl_max_cooldown_secs`, and `rl_max_attempts` if needed.
