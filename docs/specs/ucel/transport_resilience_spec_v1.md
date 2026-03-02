# UCEL Transport Resilience Spec v1

## State machine (SSOT)
`Connecting -> Online -> Degraded -> Backoff -> CircuitOpen -> Closed`

- `Connecting`: socket dial / auth handshake.
- `Online`: heartbeat healthy and message flow active.
- `Degraded`: transient fault (idle/stale/429 burst/backpressure pressure).
- `Backoff`: reconnect delay with exponential backoff + jitter.
- `CircuitOpen`: storm guard tripped; reconnect trial is paused.
- `Closed`: graceful shutdown completed.

## Reconnect policy
- Exponential backoff with deterministic jitter (seeded xorshift).
- Storm window counts failures in recent `N` seconds.
- If threshold exceeded, transition to `CircuitOpen`.
- Circuit behavior: `Open -> HalfOpen(trial) -> Closed(on success)`.

## Heartbeat / stale / resubscribe
- Track `last_recv` timestamp per connection.
- If idle exceeds `idle_timeout`, mark stale and reconnect.
- Venue rules may require ping/pong or app-level heartbeat.
- After reconnect, active subscriptions are requeued to pending and resubscribed.

## Rate limit policy
- Respect `Retry-After` (forced gate).
- Bucketed pacing per priority class: control/private/public.
- Private stream is prioritized via dedicated bucket.
- Throttle counters MUST be observable (events per class + forced gate hits).

## Backpressure policy
- Queue has fixed cap and one overflow policy:
  - `DropNewest`
  - `DropOldest`
  - `SlowDown`
  - `SpillToDisk` (fallback to slowdown when journal unavailable)
- Observability minimum:
  - queue depth
  - dropped frames
  - spilled frames
  - slowdown events

## Graceful shutdown contract
Order MUST be preserved:
1. send close frame (best effort)
2. flush/drain buffered outbound
3. requeue active subscriptions to pending
4. join writer/reader tasks with timeout
5. optional WAL flush

Failures are surfaced as `Err`, no panic/unwrap-driven crash paths.

## Required observable counters
- reconnect_count
- circuit_open_count
- dropped_frames
- queue_depth
- throttle_events
