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
