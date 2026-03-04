# I: Execution (OMS/EMS) Contract

## Single egress
- Live send (new/replace/cancel/flatten) MUST go through the outbox worker pipeline.
- Any bypass is forbidden.

## Durable pipeline
- outbox: enqueue -> (gate) -> send -> ack/retry
- inbox: dedupe incoming events/snapshots (future extension)
- event log: append-only records for all critical transitions.

## Scheduling invariants
- Lane0 (cancel/flatten) is always prioritized and must not be starved by Lane2 (new order).

## Unknown handling
- Unknown state is allowed temporarily, but must converge via reconcile.
- Critical mismatches must recommend CANCEL_ONLY/HALT (wired via J/E in later tasks).

## Gate order
- Before send: J policy decision MUST allow.
- (Next task) E lease/audit chain must be enforced as well.
