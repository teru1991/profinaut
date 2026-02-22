# Symbol Master Runtime Wiring

Runtime entrypoint: `src/main.rs`

- `supervisor.rs`: exchange-level bulkhead boundary and per-worker health ownership.
- `rest_worker.rs`: polling path and degraded transition when rest fails.
- `ws_worker.rs`: ws lagged/reconnect signal handling.
- `resync.rs`: stale-on-restore and clear-on-fresh-snapshot behavior.
- `persistence.rs`: restore/save bridge for startup and periodic snapshots.
- `metrics.rs` / `health.rs`: operational observability state.
