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

## Task B — Descriptor Execution Engine

**Status:** DONE
**Start:** 2026-02-18
**Completed:** 2026-02-18
**Branch:** `claude/descriptor-execution-engine-LxX6F`
**Required Locks:** LOCK:services-marketdata, LOCK:docs-progress-dod
**Released Locks:** LOCK:services-marketdata, LOCK:docs-progress-dod

### Deliverables Checklist

- [x] B1 — Safe Templating DSL (parser + AST + interpreter)
- [x] B2 — Placeholder Substitution Engine
- [x] B3 — JSON Pointer Extraction + Casting Utils (RFC6901)
- [x] B4 — Safe Mini-Expr Evaluator
- [x] B5 — Maps Loader + Normalization
- [x] B6 — Public API Surfaces (for later tasks)
- [x] B7 — Unit Tests (all mandated cases)
- [x] B8 — Doc Updates (descriptor reference)

---

## Existing Implementation Audit (Task B)

### What Exists

| Path | Description | Matches Task B Spec? |
|------|-------------|----------------------|
| `services/marketdata-rs/crypto-collector/src/descriptor.rs` | Descriptor model (ExchangeDescriptor, ParseSection, ExprSettings, MapsSection, Subscription). Stores `generator`, `parse.expr`, `maps` as data only. | Yes — Task A models. Task B adds execution. |
| `services/marketdata-rs/crypto-collector/src/config.rs` | CollectorConfig with ExchangeInstance (symbols, channels). | Yes — provides runtime context for DSL. |
| `services/marketdata/app/silver/normalizer.py` | Python normalization (classify_envelope). | No — Python, different concern. Leave untouched. |

### What Does NOT Exist (Task B must create)

- No DSL tokenizer/parser/interpreter
- No placeholder substitution engine
- No JSON Pointer (RFC 6901) extraction implementation
- No expression evaluator (mini-expr)
- No TOML symbol map file loader
- No subscription generation logic
- No metadata extraction logic

### Refactor Plan

- **Keep:** All Task A code untouched (models, validation, config, health, state).
- **Add:** New modules inside `services/marketdata-rs/crypto-collector/src/`:
  - `dsl.rs` — DSL tokenizer + parser + AST + interpreter (B1)
  - `placeholder.rs` — Placeholder substitution engine (B2)
  - `json_pointer.rs` — JSON pointer extraction + casting (B3)
  - `mini_expr.rs` — Mini-expr parser + evaluator (B4)
  - `maps.rs` — Maps loader + normalization (B5)
  - `engine.rs` — Public API surfaces (B6)
- **Modify:** `main.rs` — add `mod` declarations only (no logic changes).
- **Modify:** `Cargo.toml` — add `uuid` dependency for placeholder `{uuid}`.

---

## Chosen Paths

| Purpose | Path |
|---------|------|
| Task B source modules | `services/marketdata-rs/crypto-collector/src/{dsl,placeholder,json_pointer,mini_expr,maps,engine}.rs` |
| Cargo manifest (dep addition) | `services/marketdata-rs/crypto-collector/Cargo.toml` |
| Progress tracking | `docs/progress/marketdata_collector_framework_v1_4.md` |
| Definition of Done | `docs/dod/marketdata_collector_framework_v1_4.md` |
| Reference docs | `docs/descriptor_reference_v1_4.md` |

---

## End-of-Task Update (Task B)

**Completed:** 2026-02-18

### Verification Results

| Step | Result |
|------|--------|
| `cargo build -p crypto-collector` | PASS — no warnings |
| `cargo fmt -p crypto-collector -- --check` | PASS — no diffs |
| `cargo clippy -p crypto-collector -- -D warnings` | PASS — no warnings |
| `cargo test -p crypto-collector` | PASS — 87/87 tests passed (15 Task A + 72 Task B) |

### Test Breakdown (Task B: 72 tests)

| Module | Tests | Coverage |
|--------|-------|----------|
| `dsl` | 11 | Nested foreach, if/else/else-if, emit, cap enforcement, syntax errors (line/col), escape handling, comments, undefined variable |
| `placeholder` | 12 | Basic substitution, now_ms, uuid format, env var success/missing, arg success/missing, unknown placeholder, validation, JSON-mixed templates |
| `json_pointer` | 13 | Nested extraction, array index, missing, RFC6901 escapes, casting (u64/i64/string/bool), numeric strings, extract_typed (required/optional/cast fail/null) |
| `mini_expr` | 13 | Dot access, array indexing, out-of-range, missing field, fallback (null/present/chain), to_number, to_string, unknown function, too-long, AST node limit, combined access, string literal |
| `maps` | 8 | Symbol/channel normalize hit/miss, file load, missing file error, channel map build, combined maps, no-files passthrough |
| `engine` | 7 | Generate subscriptions (basic/no-placeholders), extract metadata (basic/missing-required/with-expr), normalize metadata (with-maps/passthrough) |

### Migration Notes

- No existing code was refactored. All new code in 6 new modules.
- Only changes to existing files: `main.rs` (mod declarations), `Cargo.toml` (uuid dep).
- Task A's 15 tests continue to pass unchanged.

### Notes for Future Tasks (C onward)

- `generate_subscriptions()` currently applies placeholder substitution post-DSL. For `{symbol}` and `{ch}` in emitted strings, the DSL emits them as raw template text. The engine substitutes `{conn_id}`, `{now_ms}`, `{uuid}`, `{env:*}`, `{arg:*}`. For symbol/channel-aware substitution, Task C should enhance the DSL interpreter to do inline substitution during foreach loops.
- `extract_metadata()` and `normalize_metadata()` are ready for integration with WS message handlers (Task C/D).
- The mini-expr engine output is `serde_json::Value`; cast to typed values using `json_pointer::cast_to_*` functions.
- Maps loader expects `symbol_map_file` relative to a base directory; callers should pass the config directory.
