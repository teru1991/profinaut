# Runbook: UCEL SSOT Integrity Gate v2

## Purpose
Keep SSOT consistent across catalog ↔ coverage ↔ crate ↔ rules ↔ examples.

## How to run locally
```bash
cd ucel
cargo test -p ucel-testkit --test ssot_integrity_gate_repo_test
```

## Common failure codes and fixes

- `COVERAGE_MISSING_FILE`: create `ucel/coverage/<venue>.yaml` (copy a similar venue as template).
- `COVERAGE_MISSING_ENTRY`: add an entry for the missing `op_id`. If unsupported, set:
  - `implemented: false`
  - `tested: false`
  - `support: not_supported`
  - `strict: false`
- `CRATE_MISSING`: create venue crate directory `ucel/crates/ucel-cex-<venue>` or remove venue catalog/coverage (policy decision).
- `RULES_MISSING`: add `ucel/crates/ucel-ws-rules/rules/<venue>.toml` (copy minimal existing rules).
- `EXAMPLE_MISSING`: add `ucel/examples/venue_smoke/<venue>.rs` (compile-only, no secrets).

## Policy notes

- “NOT SUPPORTED must be explicit” (never omit entries).
- Strict venues may only use `not_supported` with `entry.strict=false`.
