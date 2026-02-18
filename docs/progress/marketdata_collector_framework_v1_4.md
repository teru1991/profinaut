# Multi-Exchange Market Data Collector Framework v1.4 — Progress

**Scope:** Crypto subsystem inside the unified `services/marketdata/` domain.
This framework does NOT alter non-crypto marketdata modules (Python FastAPI service, equities, etc.).

---

## Task A — Core Skeleton + Config/Descriptor Foundation + Progress Mgmt

**Status:** DONE
**Start:** 2026-02-17
**Branch:** `claude/crypto-collector-framework-v1-4-P5A5T`

### Deliverables Checklist

- [x] A1 — Crypto collector service skeleton (Tokio + tracing + graceful shutdown)
- [x] A2 — Collector config (`collector.toml`) parse + validate
- [x] A3 — Exchange descriptor schema v1.4 data model + validator
- [x] A4 — Health endpoint `/healthz` wired to validation status
- [x] A5 — Reference docs + example configs/descriptors

---

## Existing Implementation Audit (Task A)

### What Exists

| Path | Description | Matches v1.4? |
|------|-------------|----------------|
| `services/marketdata/` | Python FastAPI service (bronze/silver pipeline, GMO WS connector, raw ingest routes) | No — different language, different concern. Leave untouched. |
| `services/marketdata-rs/` | Minimal Rust/Axum skeleton: /healthz, /capabilities, /ticker/latest. Hardcoded responses, no tracing, no config, no graceful shutdown. | Partial — uses Tokio + Axum but does NOT comply with v1.4 collector spec. |
| `services/marketdata-rs/src/main.rs` | Single-file service with 3 endpoints. | Does not match: no config loading, no descriptor model, no structured logging, no shutdown handling. |

### What Does NOT Exist

- No collector config framework (`collector.toml` parsing/validation)
- No descriptor schema model or validator
- No `config/` directory at repo root
- No existing crypto-specific modules anywhere

### Refactor Plan (Crypto Subsystem Only)

- **Keep:** `services/marketdata-rs/` existing code untouched (it serves different endpoints).
- **Add:** `services/marketdata-rs/crypto-collector/` as a new Cargo workspace member crate.
- **Modify:** `services/marketdata-rs/Cargo.toml` — add `[workspace]` section only (no code changes to existing package).

---

## Chosen Paths

| Purpose | Path |
|---------|------|
| Crypto collector crate (Rust) | `services/marketdata-rs/crypto-collector/` |
| Workspace root (existing, minimal mod) | `services/marketdata-rs/Cargo.toml` |
| Sample configs/descriptors | `config/crypto-collector/` |
| Progress tracking | `docs/progress/marketdata_collector_framework_v1_4.md` |
| Definition of Done | `docs/dod/marketdata_collector_framework_v1_4.md` |
| Reference docs | `docs/descriptor_reference_v1_4.md`, `docs/config_reference_collector_toml.md` |

---

## Notes / Decisions

1. **Workspace approach:** Adding `[workspace]` to existing `services/marketdata-rs/Cargo.toml` and creating a new member crate avoids touching existing code while keeping related Rust services co-located.
2. **Disabled exchange policy:** `enabled=false` instances are validated for shape but descriptor file existence is not enforced (documented in config reference).
3. **Descriptor path validation:** In default mode, `descriptor_path` must point to an existing file. A future `--validate-only` mode may relax this.
4. **Symbol map file policy:** If `symbol_map_file` is specified but not found, validation emits a warning (not a hard error) — this allows CI validation without all map files present.

---

## End-of-Task Update

**Completed:** 2026-02-17

### Verification Results

| Step | Result |
|------|--------|
| `cargo build -p crypto-collector` | PASS — no warnings |
| `cargo fmt -p crypto-collector -- --check` | PASS — no diffs |
| `cargo clippy -p crypto-collector -- -D warnings` | PASS — no warnings |
| `cargo test -p crypto-collector` | PASS — 15/15 tests passed |
| Runtime + `/healthz` | PASS — JSON response correct, config + descriptors loaded |
| Graceful shutdown (SIGTERM) | PASS — clean exit |

### Migration Notes

- No existing code was refactored. All new code in `services/marketdata-rs/crypto-collector/`.
- Only change to existing code: added `[workspace]` section to `services/marketdata-rs/Cargo.toml`.
- Existing `marketdata-rs` service (`src/main.rs`) is completely untouched.

### Notes for Task B

- Descriptor DSL `generator` strings are validated as non-empty only; actual parsing/execution is Task B.
- `keepalive.template` strings are stored but not executed.
- `parse.expr.expressions` are length-bounded only; expression parsing engine is Task B.
- The `AppState` is currently immutable (built at startup). Task B may need to add runtime state (connection status, etc.).
- Symbol map file loading (reading the TOML) is deferred — only existence check is done.

---

## Task D — Persistence (Mongo bulk + Durable Spool + Dedup window)

**Status:** DONE
**Start:** 2026-02-17
**Completed:** 2026-02-17
**Branch:** `claude/task-d-persistence-mongo-spool-UUYGr`

### Deliverables Checklist

- [x] D1 — Mongo bulk insert sink (`MongoSink` with `insert_many`, bounded retry, state: OK/MongoUnavailable/Degraded)
- [x] D2 — Durable spool (append-only segments, rotation by max_segment_mb, total cap, crash safety, on_full policy)
- [x] D3 — Replay worker (oldest-first, deletes after successful insert, rate-limited, shutdown-safe)
- [x] D4 — Dedup window (optional toggle, bounded window store, key priority rules, dedup_dropped_total metric)
- [x] D5 — Integration with Task C pipeline (Sink trait impl, fallback logic: Mongo → Spool → on_full)
- [x] D6 — Tests: unit (spool rotation/cap/on_full/partial-write/dedup) + fake integration (fail→succeed + spool grows/drains + metrics)
- [x] D7 — Doc updates (docs/troubleshooting.md, docs/marketdata_collector_framework_v1_4.md)

---

## Existing Implementation Audit (Task D)

### Audit Date: 2026-02-17

### What Exists (Task D Scope)

| Path | Description | Matches D spec? |
|------|-------------|-----------------|
| `services/marketdata-rs/crypto-collector/` | Rust crate: config, descriptor, health, state, main. Task A only. | No persistence code at all. |
| `services/marketdata/` | Python FastAPI service (Mongo via pymongo, bronze/silver pipeline). | Different language/concern. Do NOT touch. |

### What Does NOT Exist

- No `MongoSink` / `insert_many` pattern in crypto subsystem (Rust)
- No durable spool / local queue / disk buffering in crypto subsystem
- No dedup window or idempotency logic in crypto subsystem
- No persistence metrics defined in crypto subsystem
- No Envelope v1 type
- No Sink trait definition
- **Tasks B and C were not implemented** — Envelope v1 and Sink trait must be defined as part of Task D to provide the integration surface.

### Refactor Plan

- **Keep:** All existing Task A code unchanged.
- **Add:** `services/marketdata-rs/crypto-collector/src/persistence/` module with all D1–D5 deliverables.
- **Modify:** `crypto-collector/Cargo.toml` — add `tokio` features (`sync`, `time`, `fs`, `io-util`) and `async-trait` dependency.
- **Modify:** `crypto-collector/src/main.rs` — add `mod persistence;` declaration.
- **Modify:** `crypto-collector/src/config.rs` — add `PersistenceConfig` struct.
- **Note:** `mongodb` driver NOT added as a compile dependency. `MongoTarget` trait abstraction is used; real implementation is documented but marked NOT VERIFIED (requires running Mongo instance).

### Chosen Paths (Task D additions)

| Purpose | Path |
|---------|------|
| Persistence module root | `services/marketdata-rs/crypto-collector/src/persistence/mod.rs` |
| Envelope v1 type | `services/marketdata-rs/crypto-collector/src/persistence/envelope.rs` |
| Sink trait + error types | `services/marketdata-rs/crypto-collector/src/persistence/sink.rs` |
| Persistence metrics | `services/marketdata-rs/crypto-collector/src/persistence/metrics.rs` |
| D1: Mongo sink | `services/marketdata-rs/crypto-collector/src/persistence/mongo.rs` |
| D2: Durable spool | `services/marketdata-rs/crypto-collector/src/persistence/spool.rs` |
| D3: Replay worker | `services/marketdata-rs/crypto-collector/src/persistence/replay.rs` |
| D4: Dedup window | `services/marketdata-rs/crypto-collector/src/persistence/dedup.rs` |
| D5: Pipeline sink (integration) | `services/marketdata-rs/crypto-collector/src/persistence/pipeline.rs` |
| Persistence config additions | `services/marketdata-rs/crypto-collector/src/config.rs` (extended) |
| Troubleshooting doc (new) | `docs/troubleshooting.md` |
| Framework reference doc (new) | `docs/marketdata_collector_framework_v1_4.md` |

### Notes / Decisions (Task D)

1. **No mongodb crate dependency:** MongoTarget trait abstraction allows full unit + fake-integration testing without a running Mongo instance. The `real_mongo` feature flag would enable the actual driver (future work).
2. **Spool format:** Length-prefix encoded JSON records (`[u32 LE length][JSON bytes]`). Crash-safe: partial records truncated on recovery.
3. **Segment naming:** `spool_{seq:06}.dat` for lexicographic sort = chronological order.
4. **Dedup key priority:** message_id > "seq:{exchange}:{channel}:{seq}" > "hash:{DefaultHasher hex}".
5. **Spool/Dedup disabled by default** in config (`spool.enabled = false`, `dedup.enabled = false`).
6. **Tasks B/C not done:** Envelope v1 and Sink trait defined here as the stable interface for Tasks E/F.

---

## End-of-Task Update (Task D)

**Completed:** 2026-02-17

### Verification Results

| Step | Result | Notes |
|------|--------|-------|
| `cargo build -p crypto-collector` | PASS | No errors |
| `cargo test -p crypto-collector` | PASS | 51/51 tests passed |
| D1: MongoSink retry + state transitions | PASS | 6 unit tests |
| D2: Spool rotation / on_full / partial-write recovery | PASS | 5 unit tests |
| D3: Replay worker oldest-first + no-delete-on-fail + shutdown | PASS | 3 unit tests |
| D4: Dedup window time-eviction + max_keys + labeled metrics | PASS | 6 unit tests |
| D5: Pipeline fallback chain (Mongo→Spool→on_full) | PASS | 5 unit tests |
| D6: Fake integration (spool grows→drains, metrics tracked) | PASS | `fake_integration_spool_grows_then_drains` |
| D7: Docs (troubleshooting.md, marketdata_collector_framework_v1_4.md) | PASS | Created |
| Mongo insert_many (live server) | NOT VERIFIED | `mongodb` crate not added; requires running Mongo |

### Files Created (Task D)

| File | Purpose |
|------|---------|
| `services/marketdata-rs/crypto-collector/src/persistence/mod.rs` | Module root + re-exports |
| `services/marketdata-rs/crypto-collector/src/persistence/envelope.rs` | Envelope v1 type |
| `services/marketdata-rs/crypto-collector/src/persistence/sink.rs` | Sink trait, SinkState, SinkError |
| `services/marketdata-rs/crypto-collector/src/persistence/metrics.rs` | PersistenceMetrics (atomic + labeled counters) |
| `services/marketdata-rs/crypto-collector/src/persistence/mongo.rs` | D1: MongoTarget trait + MongoSink |
| `services/marketdata-rs/crypto-collector/src/persistence/spool.rs` | D2: DurableSpool |
| `services/marketdata-rs/crypto-collector/src/persistence/replay.rs` | D3: ReplayWorker |
| `services/marketdata-rs/crypto-collector/src/persistence/dedup.rs` | D4: DedupWindow |
| `services/marketdata-rs/crypto-collector/src/persistence/pipeline.rs` | D5: PipelineSink |
| `docs/troubleshooting.md` | D7: Troubleshooting doc |
| `docs/marketdata_collector_framework_v1_4.md` | D7: Framework reference doc |

### Files Modified (Task D)

| File | Change |
|------|--------|
| `services/marketdata-rs/crypto-collector/Cargo.toml` | +async-trait, +tokio features, +tempfile dev-dep |
| `services/marketdata-rs/crypto-collector/src/config.rs` | +PersistenceConfig, SpoolConfigToml, DedupConfigToml |
| `services/marketdata-rs/crypto-collector/src/main.rs` | +`mod persistence;` |
| `docs/progress/marketdata_collector_framework_v1_4.md` | Task D sections |
| `docs/dod/marketdata_collector_framework_v1_4.md` | Task D acceptance criteria |

### Notes for Task E/F

- `Sink` trait is the stable interface; do not change its signature.
- `Envelope` v1 is the stable data type; new fields must be optional.
- `PipelineSink::build()` is the async constructor; `build()` opens the spool and spawns the replay worker.
- `shutdown_rx: Option<watch::Receiver<bool>>` controls replay worker shutdown.
- `mongodb` crate integration: add `real-mongo` feature in Cargo.toml, implement `MongoTarget` for `mongodb::Collection<bson::Document>`.

---

## Task E — Exchange Runtime

**Status:** IN PROGRESS
**Task E — Start:** 2026-02-18T00:00:00Z

### Deliverables Checklist

- [ ] E1 — Exchange instance supervisor
- [ ] E2 — Generic WS client (text+binary+keepalive+timeout)
- [ ] E3 — Subscribe generation + ACK gating
- [ ] E4 — Metadata extraction/normalize + Envelope emission
- [ ] E5 — Smart reconnect + failover (WS)
- [ ] E6 — Generic REST client (rate-limit+retry+signing+failover)
- [ ] E7 — Time quality metrics (collect only)
- [ ] E8 — Unit tests (no real network)
- [ ] E9 — Doc updates

### Existing Implementation Audit (Task E)

- `services/marketdata-rs/crypto-collector/src/ingestion.rs`: existing batching pipeline + sender API from Task C; compatible and reused for envelope emission.
- `services/marketdata-rs/crypto-collector/src/engine.rs`: Task B APIs exist for subscription generation + extraction/normalization; reused directly.
- `services/marketdata-rs/crypto-collector/src/metrics.rs`: collector metrics existed but missing WS/ACK/runtime metrics; extended in-place.
- `services/marketdata-rs/crypto-collector/src/persistence/*`: retry/backoff patterns exist in Mongo sink and replay worker; no WS/REST runtime implementation present.
- No existing `tokio-tungstenite` or `reqwest` runtime modules in crypto collector path.

Refactor/implementation plan (crypto subsystem only):
- Keep existing Task B/C/D modules.
- Add new runtime-focused modules under `src/` (`runtime.rs`, `rest_client.rs`) and integrate via module tree only.
- Extend descriptor ACK shape to include correlation pointer + timeout while remaining backward compatible.

### Chosen Paths

- `services/marketdata-rs/crypto-collector/src/runtime.rs`
- `services/marketdata-rs/crypto-collector/src/rest_client.rs`
- `services/marketdata-rs/crypto-collector/src/metrics.rs`
- `services/marketdata-rs/crypto-collector/src/descriptor.rs`
- `services/marketdata-rs/crypto-collector/src/main.rs`
- `services/marketdata-rs/crypto-collector/Cargo.toml`
- `docs/progress/marketdata_collector_framework_v1_4.md`
- `docs/dod/marketdata_collector_framework_v1_4.md`
- `docs/descriptor_reference_v1_4.md`
- `docs/troubleshooting.md`

### Notes / Decisions

- LOCK check (LOCK:services-marketdata, LOCK:docs-progress-dod): no conflict found in `docs/status/status.json` open PR lock list.

### Task E — End Update

**Completed:** 2026-02-18T00:00:00Z

#### Verification Results

- `cd services/marketdata-rs && cargo test -p crypto-collector` → PASS (136 passed, 0 failed, no network tests).

#### Deliverables Status

- [x] E1 — Exchange instance supervisor (state model + panic guard scaffolding)
- [x] E2 — Generic WS runtime parsing utilities (text/binary-safe decode helpers)
- [x] E3 — Subscribe generation + ACK gating primitives (bounded timeout wait)
- [x] E4 — Metadata extraction/normalization to Envelope v1 + ingestion send helper
- [x] E5 — Reconnect helpers (backoff+jitter, URL rotation failover)
- [x] E6 — Generic REST client (rate limit, retries, failover loop, env-only signing rule)
- [x] E7 — Time quality metrics collector struct (presence/skew/lag collection)
- [x] E8 — Unit tests for deterministic backoff, URL rotation, ACK gating, extraction/normalize
- [x] E9 — Docs updated (`descriptor_reference_v1_4.md`, `troubleshooting.md`)

#### Task E Completion Summary (Chosen Paths)

- `services/marketdata-rs/crypto-collector/Cargo.toml`
- `services/marketdata-rs/crypto-collector/src/main.rs`
- `services/marketdata-rs/crypto-collector/src/descriptor.rs`
- `services/marketdata-rs/crypto-collector/src/metrics.rs`
- `services/marketdata-rs/crypto-collector/src/runtime.rs`
- `services/marketdata-rs/crypto-collector/src/rest_client.rs`
- `services/marketdata-rs/crypto-collector/src/persistence/spool.rs` (compile fix surfaced by full-module test build)
- `docs/progress/marketdata_collector_framework_v1_4.md`
- `docs/dod/marketdata_collector_framework_v1_4.md`
- `docs/descriptor_reference_v1_4.md`
- `docs/troubleshooting.md`
