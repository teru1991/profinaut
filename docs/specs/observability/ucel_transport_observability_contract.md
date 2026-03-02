# UCEL Transport Observability Contract (SSOT)

## Scope
This document fixes the **minimum observability contract** provided by `ucel-transport`.
Downstream components must rely on this contract (stable names/labels/units).

## Structured logging: REQUIRED keys
Every transport log/event/span MUST include:
- `exchange_id` (string)
- `conn_id` (string)
- `op` (string; UCEL op name)
- `symbol` (string; canonical symbol or "*" if not applicable)
- `run_id` (string; correlation across a run)

The library provides helpers that **make missing keys impossible** at the callsite:
- `ucel_transport::obs::logging::{ObsRequiredKeys, span_required}`

## Metrics: UCEL standard set (minimum)
Counters (monotonic):
- `ucel_transport_reconnect_attempts_total`
- `ucel_transport_reconnect_success_total`
- `ucel_transport_reconnect_failure_total`
- `ucel_transport_breaker_open_total`
- `ucel_transport_stale_requeued_total`
- `ucel_transport_outq_dropped_total`
- `ucel_transport_outq_spilled_total`
- `ucel_transport_rl_penalty_applied_total`
- `ucel_transport_rl_cooldown_set_total`
- `ucel_transport_deadletter_total`

Gauges:
- `ucel_transport_outq_len`
- `ucel_transport_wal_queue_len`
- `ucel_transport_last_inbound_age_ms`

## Prometheus export
`ucel-transport` provides:
- `ucel_transport::obs::export_prometheus::encode_prometheus_text(...)`

The outer process can expose:
- `GET /metrics` -> returns this text.

## Stability events ring
Transport keeps recent stability events (secret-free) for supportability:
- reconnect, breaker open, overflow/drop/spill, rate-limit penalty/cooldown, etc.

(Full event taxonomy will be fixed in Step2/Step3 with crosscut support_bundle and C-level outline.)
