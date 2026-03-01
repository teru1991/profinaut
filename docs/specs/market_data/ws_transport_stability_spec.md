# WS Transport Stability Spec (SSOT)

## Purpose
ucel-transport（WS）を「落ちない・復帰する・観測できる・診断できる」状態に固定する。  
実装はこのSpecを正本（SSOT）として従う。

## Scope
- reconnect: backoff+jitter / storm guard / circuit breaker（Open→HalfOpen→Close）
- heartbeat: idle timeout → reconnect
- subscription stale: stale sweep → pending requeue → auto resubscribe
- rate-limit nack: Retry-After尊重、penalty + cooldown、max_attempts で暴走停止
- backpressure: overflow policy（Drop/SlowDown/Spill-to-disk）
- graceful shutdown: close→flush→requeue→join（abortはtimeout後のみ）

## Definitions
- **Idle**: last inbound message age >= idle_timeout
- **Stale subscription**: last_message_at(or first_active_at) < now - stale_after
- **Rate-limit nack**: nack reason が rate/limit/throttle/429 等、または retry_after_ms を伴う

## Policy
### Reconnect
- storm guard: window=W, max=N を超えたら停止（運用判断へ）
- breaker:
  - Closed: failures < threshold
  - Open: cooldown 中は接続試行しない
  - HalfOpen: trial は最大T、success S 回で Closed、failure で Open

### Rate-limit handling
- retry_after_ms があれば優先
- 無い場合の default_penalty_ms を適用（rulesで上書き可能）
- RL nack を受けた subscription は:
  - max_attempts を超えたら deadletter
  - それ以外は pending + rate_limit_until=now+cooldown（cooldownは指数backoff）

### Overflow/backpressure
- policy: DropNewest / DropOldestLowPriority / SlowDownThenDropOldestLowPriority / SpillToDiskThenDropOldestLowPriority
- Spill の I/O 失敗時は fallback（DropOldestLowPriority）へ

### Graceful shutdown
- close request enqueue（control）
- drain outbound/wal
- requeue active/inflight → pending
- join tasks（timeout後のみabort）

## Observability (minimum)
- metrics (counters): reconnect_attempts, reconnect_failure, breaker_open, stale_requeued, outq_dropped, outq_spilled, rl_penalty_applied, rl_cooldown_set, deadletter_count
- gauges: outq_len, wal_queue_len, last_inbound_age_ms
- events tail: reconnect_reason, breaker_state, overflow outcome, stale sweep result, rl cooldown/penalty

## Compatibility
- rules.toml は後方互換（[stability] は optional）
- 不正値は loader validation で reject

## Acceptance (DoD)
- 001〜004 が完了し、runbooks + support_bundle + /healthz が揃って運用できる
