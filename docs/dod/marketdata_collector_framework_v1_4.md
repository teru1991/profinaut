# Definition of Done — Multi-Exchange Market Data Collector Framework v1.4

## Task A — Core Skeleton + Config/Descriptor Foundation

### Acceptance Criteria

- [x] A1: Service skeleton starts with Tokio multi-thread runtime, structured tracing, graceful shutdown (SIGINT/SIGTERM)
- [x] A2: `collector.toml` config loads and validates with contextual error messages per exchange instance
- [x] A3: Descriptor v1.4 schema parses and validates (required fields, unique connection IDs, subscription cross-refs, pointer format)
- [x] A4: `/healthz` endpoint returns JSON with service, version, config_loaded, descriptors_loaded_count, per-instance status
- [x] A4b: `/healthz` works even when config/descriptors have errors (reports them)
- [x] A5: Reference docs exist for descriptor schema and collector.toml
- [x] A5b: Sample configs/descriptors validate successfully
- [x] Chosen Paths are recorded in progress doc
- [x] No changes to non-crypto modules (Python marketdata service, existing marketdata-rs endpoints)

### Verification Steps

| Step | Command | Expected |
|------|---------|----------|
| Build | `cd services/marketdata-rs && cargo build -p crypto-collector` | Success |
| Format | `cd services/marketdata-rs && cargo fmt -p crypto-collector -- --check` | No diffs |
| Clippy | `cd services/marketdata-rs && cargo clippy -p crypto-collector -- -D warnings` | No warnings |
| Unit tests | `cd services/marketdata-rs && cargo test -p crypto-collector` | All pass |
| Run + healthz | Start service, `curl http://127.0.0.1:<port>/healthz` | JSON with expected fields |
| Graceful shutdown | Send SIGINT to running service | Clean exit, no panics |
| Config error reporting | Modify config to have duplicate exchange names, run | Error message names the duplicate |
| Descriptor error reporting | Modify descriptor to have invalid connection_id ref | Error message identifies the bad reference |

### Verification Results

| Step | Result | Notes |
|------|--------|-------|
| Build | PASS | No warnings |
| Format | PASS | No diffs |
| Clippy | PASS | `-D warnings` — zero warnings |
| Unit tests | PASS | 15/15 tests passed (6 config, 9 descriptor) |
| Run + healthz | PASS | Returns correct JSON with all required fields |
| Graceful shutdown | PASS | SIGTERM → clean "shut down gracefully" log |
| Config error reporting | PASS | Unit test `reject_duplicate_names` verifies contextual messages |
| Descriptor error reporting | PASS | Unit test `reject_invalid_connection_ref` verifies cross-ref errors |
