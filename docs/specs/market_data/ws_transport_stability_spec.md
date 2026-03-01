# WS Transport Stability Spec

This spec defines WS stability behavior for transport:

- Rate-limit Nack MUST apply limiter penalty and cooldown.
- Retry-After hint (ms) takes precedence for cooldown.
- Cooldown rows MUST remain pending but be excluded from pending batch selection until expiry.
- Attempts MUST be bounded by `rl_max_attempts`; overflow MUST deadletter to stop loops.
- Stability must be observable via health status, metrics, and events tail.
- Support bundle MUST include transport health, metrics, event tail, and redacted rules snapshot.

## Health semantics
- `Unknown`: no stable signal yet.
- `Degraded`: reconnect storm, breaker-open, stale requeue spikes, or sustained RL penalty.
- `Unhealthy`: connection cannot be established within configured guardrails.

## Chaos test gates
- RL nack (`429`/`rate limit`) -> cooldown set + pending batch skip.
- Cooldown expiry -> pending batch picks item again.
