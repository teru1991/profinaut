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

---

## Task D — Persistence (Mongo bulk + Durable Spool + Dedup window)

### Acceptance Criteria

- [ ] D1: `MongoSink<T: MongoTarget>` exists; consumes `Vec<Envelope>` via `insert_many`; bounded retry with exponential backoff; deterministic state transitions OK→MongoUnavailable→Degraded (after N consecutive batch failures).
- [ ] D1b: Metrics: `write_batch_latency_ms` records last batch write duration; `ingest_errors_total{exchange}` increments on sink failure per exchange.
- [ ] D2: `DurableSpool` exists; append-only, length-prefix encoded, crash-safe (partial records truncated on recovery); segment rotation by `max_segment_mb`; total cap by `max_total_mb`; `on_full` policy enforced deterministically (drop_ticker_depth_keep_trade / drop_all / block).
- [ ] D2b: Metrics: `spool_bytes` (gauge), `spool_segments` (gauge), `spool_dropped_total{exchange,channel}` increments on drop, `spool_replay_total` increments on replay.
- [ ] D3: `ReplayWorker` exists; replays oldest segments first; deletes segment only after successful insert; rate-limited between batches; shutdown-safe via watch channel.
- [ ] D4: `DedupWindow` exists; bounded (window_seconds + max_keys); key priority: message_id > seq > hash; `dedup_dropped_total{exchange,channel}` increments correctly; no memory leak.
- [ ] D5: `PipelineSink` implements `Sink` trait; fallback chain: Mongo OK → write; Mongo unavailable + spool enabled → spool; spool full → apply on_full policy; interfaces stable for Task E/F.
- [ ] D6: Unit tests exist and pass: spool rotation, spool cap, on_full policies, partial write recovery, dedup window and eviction. Fake integration test exists and passes: fake sink fails then succeeds, spool grows then drains, metrics change.
- [ ] D7: `docs/troubleshooting.md` updated with Mongo-down + spool behavior. `docs/marketdata_collector_framework_v1_4.md` updated with durability and dedup semantics.
- [ ] Chosen Paths recorded in progress doc and completion summary.
- [ ] No changes to non-crypto modules or forbidden paths.

### Verification Steps

| Step | Command | Expected |
|------|---------|----------|
| Build | `cd services/marketdata-rs && cargo build -p crypto-collector` | Success, no warnings |
| Unit tests | `cd services/marketdata-rs && cargo test -p crypto-collector` | All pass |
| Spool rotation test | Covered in unit tests | Verify segment file rotation at max_segment_mb |
| Spool cap / on_full test | Covered in unit tests | Verify on_full policy applied when total cap exceeded |
| Partial write recovery test | Covered in unit tests | Verify truncation of incomplete last record |
| Dedup eviction test | Covered in unit tests | Verify entries evicted by time and by max_keys |
| Fake integration test | Covered in unit tests | Spool grows on Mongo failure, drains on Mongo recovery, metrics correct |
| Mongo insert_many | NOT VERIFIED — requires live Mongo instance | See troubleshooting.md for manual steps |

### Verification Results

| Step | Result | Notes |
|------|--------|-------|
| Build | NOT VERIFIED | Pending |
| Unit tests | NOT VERIFIED | Pending |
| Spool rotation | NOT VERIFIED | Pending |
| Spool cap / on_full | NOT VERIFIED | Pending |
| Partial write recovery | NOT VERIFIED | Pending |
| Dedup eviction | NOT VERIFIED | Pending |
| Fake integration | NOT VERIFIED | Pending |
| Mongo insert_many | NOT VERIFIED | Requires live Mongo; mongodb crate not added as compile dep |
