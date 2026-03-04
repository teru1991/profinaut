# Observability Metrics Catalog (C-3 / C-5-3)

This document is the SSOT for Prometheus metrics that must be exposed by major Python services.

## Naming and unit rules

- Prefix: `profinaut_`
- Format: `profinaut_<domain>_<metric>` in `snake_case`
- Units:
  - durations use `_seconds`
  - sizes use `_bytes`
  - counters use `_total`
  - gauges with state flags use one-hot labels where applicable

## Required common metrics

1. `profinaut_build_info{service,version,git_sha}` gauge (always 1)
2. `profinaut_uptime_seconds{service}` gauge
3. `profinaut_health_status{service,status}` gauge (`status`: `OK|DEGRADED|FAILED|UNKNOWN` one-hot)
4. `profinaut_capabilities_present{service}` gauge (`1` when `/capabilities` exists)
5. `profinaut_http_requests_total{service,op,method,status_class}` counter
6. `profinaut_http_request_duration_seconds{service,op,method}` histogram

## Domain minimum metrics

- execution: `profinaut_execution_orders_total{service,result}`
- marketdata: `profinaut_marketdata_frames_total{service,venue,result}`

## Label policy (low cardinality)

Allowed labels (fixed sets only):

- `service`, `op`, `method`, `status_class`, `result`, `venue`, `status`, `version`, `git_sha`

Forbidden labels:

- `symbol`, `order_id`, `client_order_id`, `price`, `qty`, `size`, `amount`, `notional`, `ip`, `host`, `hostname`, `endpoint`, `url`, `trace_id`, `run_id`

## Compatibility and gate

- Breaking changes (metric rename/type/required-label changes) require catalog versioning.
- `docs/contracts/observability/metrics_catalog.snapshot.txt` is CI-gated and must match generated output.
