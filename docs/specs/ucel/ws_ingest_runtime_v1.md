# UCEL WS Ingest Runtime v1

## Lifecycle
Planned -> PendingConnect -> Connecting -> AwaitingAuth/AwaitingAck -> Active -> StallSuspected -> ReconnectScheduled -> ResumePending -> PendingConnect.
Terminal states: Deadlettered, Drained, Completed.

## Ownership
- planner: desired plan source of truth.
- store: durable runtime state source of truth.
- journal: append-only evidence source of truth.

## Journal-first
State transitions and failures (ack timeout, heartbeat timeout, gap/checksum mismatch, rate-limit, shutdown) are appended to journal before store update.

## Resume
On restart, resume input is reconstructed from durable store + journal replay.
Private streams require reauth; public streams require resubscribe (+resnapshot where integrity demands).
