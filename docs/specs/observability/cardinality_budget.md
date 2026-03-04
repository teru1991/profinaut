# Observability Cardinality and Budget Guard (C-17 / C-16 / C-3)

## High-cardinality risk examples

Fields/labels such as `symbol`, `order_id`, `price`, `trace_id`, `run_id` can rapidly increase unique time-series and log fanout.

## Low-cardinality label rule

Only stable low-cardinality label dimensions should be used for metrics. Dynamic per-event identifiers must not be promoted to metric labels.

## Runtime defense behavior

1. Pre-check: guard validates new labelsets/field keys against configured budgets.
2. Suppression: when exceeded, policy applies `drop` / `aggregate` / `sample` (metrics) and `truncate` / `drop` (logs).
3. Incident observability: violations update counters/gauges, emit audit events, and degrade health/capabilities status.

## Budget exceed signaling

When observability budget is exceeded, health/capabilities surfaces `DEGRADED` with `OBS_BUDGET_EXCEEDED`.
