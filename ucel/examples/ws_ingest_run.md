# ws-ingest minimal run

```bash
cd ucel
cargo run -p ucel-ws-ingest
```

Single exchange allowlist is currently configured in code via `IngestConfig.exchange_allowlist`.

Operational safety baseline in this phase:
- reconnect by upper-layer policy
- backpressure stop-first behavior
- deadletter recording handled by durable store API
