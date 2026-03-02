# UCEL SSOT Integrity Gate v2 (Catalog ↔ Coverage ↔ Crate ↔ Rules ↔ Examples)

- Task ID: `UCEL-SSOT-GATE-V2-001`
- Status: Draft → Approved by merge
- Scope: SSOT/Contracts/Gate (Crosscut)

## 1. Purpose

This document fixes the v2 invariants for SSOT consistency across:

- Catalog SSOT: `docs/exchanges/<venue>/catalog.json`
- Coverage SSOT (v1 legacy + v2 future): `ucel/coverage/*.yaml` (and `ucel/coverage_v2/*.yaml`)
- Crates: `ucel/crates/ucel-cex-<venue>`
- WS Rules: `ucel/crates/ucel-ws-rules/rules/*.toml`
- Examples: `ucel/examples/**` (venue smoke examples)

Goal: make “unsupported” explicit in SSOT and enable fail-fast CI gate.

## 2. Coverage v1 schema (backward-compatible extension)

Existing v1 coverage format:

```yaml
venue: okx
strict: true
entries:
  - id: crypto.public.ws.ticker
    implemented: true
    tested: false
```

v2-compatible extension adds:

- `support: supported | not_supported` (default: `supported`)
- `strict: true|false` (optional per-entry override; default: absent → follow `manifest.strict`)

Example:

```yaml
venue: okx
strict: true
entries:
  - id: crypto.public.ws.orderbook.l2
    implemented: true
    tested: true
    support: supported
    strict: true
  - id: crypto.public.ws.some_future_channel
    implemented: false
    tested: false
    support: not_supported
    strict: false
```

### 2.1 Effective strictness

Define:

- `manifest.strict`: venue default strictness
- `entry.strict`: optional override

`effective_strict(entry) = entry.strict.unwrap_or(manifest.strict)`

## 3. “NOT SUPPORTED must be explicit” rule

If an operation exists in catalog but is not supported by implementation, coverage MUST contain an entry with:

- `support: not_supported`
- `implemented: false` (recommended)
- `tested: false` (recommended)
- `strict: false` (recommended; prevents strict failure for known unsupported)

Missing an entry is forbidden under the integrity gate (v2): lack of explicit NOT SUPPORTED is treated as an error.

## 4. Semantics for existing gates

Legacy coverage gate that checks implemented/tested MUST ignore entries with `support: not_supported`:

- They are not “missing implementation”; they are “explicitly unsupported”.

This keeps CI meaningful and avoids false negatives.

## 5. Gate v2 responsibilities (high-level)

Integrity Gate v2 will verify:

- Catalog ↔ Coverage: every catalog op has a coverage entry (supported or not_supported).
- Coverage ↔ Crate: if venue is in catalog/coverage, its crate must exist (unless explicitly allowlisted by future policy).
- Coverage ↔ Rules: strict venues must have at least one matching rules file.
- Coverage ↔ Examples: strict venues must have a venue smoke example.

(Implementation is Task UCEL-SSOT-GATE-V2-002.)

## 6. Compatibility and migration plan

- Existing coverage YAML without support/strict remains valid.
- Phase 1: introduce fields + docs + ignore-not_supported logic (this task).
- Phase 2: introduce integrity gate v2 and make missing entries fail.
- Phase 3: fill coverage with explicit NOT SUPPORTED entries to pass gate v2.
