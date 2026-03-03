# UCEL Observability Spec v1

## Required keys
All UCEL logs and trace spans MUST include:
- `exchange_id`
- `conn_id`
- `op`
- `symbol`
- `run_id`
- `trace_id`
- `request_id`

## Standard metrics
- `ucel_ws_reconnect_total`
- `ucel_ws_frames_total`
- `ucel_ws_decode_errors_total`
- `ucel_ws_queue_depth`
- `ucel_ws_wal_write_latency_ms`
- `ucel_ws_throttle_events_total`
- `ucel_ws_circuit_open_total`
- `ucel_ws_dropped_frames_total`

## Span design
- `ucel_ws_connection` span: `exchange_id`, `conn_id`, `run_id`, `trace_id`, `request_id`
- `ucel_ws_operation` span: `exchange_id`, `conn_id`, `op`, `symbol`, `run_id`, `trace_id`, `request_id`

## Logging level policy
- `warn`: auto-recoverable conditions (reconnects, throttling, queue pressure)
- `error`: non-recoverable conditions, classified with `reason`
