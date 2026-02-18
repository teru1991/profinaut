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

## Task B — Descriptor Execution Engine

### Acceptance Criteria

- [x] B1: DSL parser produces AST with line/column error attribution; interpreter supports nested foreach, if/else, emit with bounded output (cap 1,000,000)
- [x] B2: Placeholder engine substitutes {symbol}, {ch}, {channel}, {conn_id}, {now_ms}, {uuid}, {env:VAR}, {arg:KEY}; errors on unknown/missing env vars
- [x] B3: JSON pointer extraction implements RFC 6901; cast_to_u64/i64/string/bool; extract_typed with optional/required semantics
- [x] B4: Mini-expr evaluator supports dot access, array indexing, ?? fallback, to_number/to_string functions; rejects arithmetic/unknown functions; enforces AST node + step bounds
- [x] B5: Maps loader reads TOML symbol_map_file; channel_map from descriptor; normalize_symbol/normalize_channel passthrough on miss
- [x] B6: generate_subscriptions(), extract_metadata(), normalize_metadata() are pub and callable without networking
- [x] B7: Unit tests cover all mandated cases (DSL, placeholder, JSON pointer, mini-expr, maps)
- [x] B8: Descriptor reference doc updated with DSL grammar, placeholder set, mini-expr limits, safety/constraints section
- [x] No changes to non-crypto modules
- [x] Chosen Paths recorded in progress doc

### Verification Steps

| Step | Command | Expected |
|------|---------|----------|
| Build | `cd services/marketdata-rs && cargo build -p crypto-collector` | Success, no warnings |
| Format | `cd services/marketdata-rs && cargo fmt -p crypto-collector -- --check` | No diffs |
| Clippy | `cd services/marketdata-rs && cargo clippy -p crypto-collector -- -D warnings` | No warnings |
| Unit tests | `cd services/marketdata-rs && cargo test -p crypto-collector` | All pass |
| DSL nested foreach | Unit test: symbols × channels produces correct count + content | Pass |
| DSL if/else | Unit test: condition-based path selection | Pass |
| DSL cap enforcement | Unit test: exceeding 1M outputs returns error | Pass |
| DSL syntax errors | Unit test: invalid input returns line/col error | Pass |
| Placeholder unknown | Unit test: unknown placeholder returns error | Pass |
| Placeholder env var | Unit test: missing env var returns error | Pass |
| JSON pointer missing required | Unit test: returns error with pointer path | Pass |
| JSON pointer optional | Unit test: returns Ok(None) | Pass |
| Mini-expr whitelist | Unit test: unknown function returns error | Pass |
| Mini-expr bounds | Unit test: very large expression returns error | Pass |

### Verification Results

| Step | Result | Notes |
|------|--------|-------|
| Build | PASS | No warnings |
| Format | PASS | No diffs |
| Clippy | PASS | `-D warnings` — zero warnings |
| Unit tests | PASS | 87/87 tests passed (15 Task A + 72 Task B) |
| DSL nested foreach | PASS | `nested_foreach_symbols_x_channels` + `output_count_correctness` |
| DSL if/else | PASS | `if_else_if_else_path_selection` + `and_or_conditions` |
| DSL cap enforcement | PASS | `cap_enforcement` — exceeding limit returns "output cap exceeded" |
| DSL syntax errors | PASS | `syntax_error_line_col` — reports line 2, col > 0 |
| Placeholder unknown | PASS | `substitute_unknown_placeholder` — "unknown placeholder '{bogus}'" |
| Placeholder env var | PASS | `substitute_env_var_missing` — "missing environment variable" |
| JSON pointer missing required | PASS | `extract_typed_required_missing_errors` — includes "/b" |
| JSON pointer optional | PASS | `extract_typed_optional_missing_returns_none` — returns None |
| Mini-expr whitelist | PASS | `unknown_function_rejected` — "unknown_fn" rejected |
| Mini-expr bounds | PASS | `expression_too_long` + `ast_node_limit` — both enforced |
