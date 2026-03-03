# UCEL Coverage Strict Policy v1

**Task**: UCEL-H-STRICT-001
**Status**: Active
**Scope**: Market Data (H) — `ucel/coverage/` (v1 schema, all CEX venues)

---

## Purpose

Define and mechanically enforce the "100% declaration" for UCEL Market Data coverage.
`strict: true` must mean not just "implemented and tested" but also "regression assets exist".

---

## Definition

`strict: true` for a coverage venue means all three conditions hold:

1. **Coverage completeness**: `implemented: true` and `tested: true` at venue level and for all
   `entries[]` (enforced by existing `ssot_gate.rs` / `run_coverage_gate`).
2. **Golden fixture existence**: at least one file exists under
   `ucel/fixtures/golden/ws/<venue>/` (enforced by `strict_golden_gate.rs`).
3. **PR gate**: missing golden fixtures cause CI to fail before merge
   (`strict_golden_gate.rs::strict_venues_must_have_golden_fixtures`).

Condition 1 without condition 2 is insufficient: a venue can have `implemented=true` and
`tested=true` in coverage metadata yet have no regression file to catch regressions.

---

## Scope

| Area | Coverage files | Policy |
|------|---------------|--------|
| Market Data (H) | `ucel/coverage/*.yaml` (v1 schema) | **This policy — all venues strict=true** |
| v2 WS families | `ucel/coverage_v2/*.yaml` | Already strict=true; golden policy is a follow-up task |
| Execution (I) | separate spec | Separate DoD |
| Onchain / Chain | separate spec | Separate DoD |

---

## Gate implementation

`ucel/crates/ucel-testkit/tests/strict_golden_gate.rs`

- Reads all `ucel/coverage/*.yaml` files.
- Collects venues where `strict: true`.
- Asserts each venue has `ucel/fixtures/golden/ws/<venue>/` containing at least one file.
- Fails with a clear error listing missing venues.

This gate runs as part of `cargo test -p ucel-testkit`.

---

## Golden fixture levels

| Level | Contents | What it proves |
|-------|----------|----------------|
| **Stub** (minimum) | `stub.json` with `{"_stub": true}` | Existence gate passes; regression asset placeholder |
| **Snapshot** (recommended) | `<case>/raw.json` + `<case>/expected.normalized.json` | Normalization is stable (tested by `golden_ws.rs`) |

New venues should progress from Stub to Snapshot as their normalizer is implemented.

---

## How to add a new venue

1. Add entry to `ucel/coverage/<venue>.yaml` with `strict: false` initially.
2. Implement the adapter and tests.
3. Create at least `ucel/fixtures/golden/ws/<venue>/stub.json`.
4. Set `strict: true` in the coverage file.
5. Confirm `cargo test -p ucel-testkit --test strict_golden_gate` passes.
6. Optionally promote the stub to a full golden snapshot.

---

## Exception process

If a venue temporarily cannot satisfy the strict requirement:
- Keep `strict: false` in coverage.
- Document the reason in `entries[].notes` in the coverage YAML.
- Track the issue in `docs/status/trace-index.json`.

Do **not** merge with `strict: true` and a broken gate.

---

## Normalization test separation

`strict_golden_gate.rs` — existence only
`golden_ws.rs` — normalization correctness for venues with subdirectory case fixtures

The two tests work together: strict_golden_gate ensures every strict venue has *something*;
golden_ws ensures the normalization output is stable for venues with case subdirectories.
