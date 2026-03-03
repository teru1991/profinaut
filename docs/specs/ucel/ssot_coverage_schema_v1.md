# UCEL SSOT Coverage Schema v1

> LEGACY NOTICE
> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
> Current SSOT is coverage_v2 under `ucel/coverage/coverage_v2/` and `ucel/docs/policies/coverage_policy.md`.

## Purpose
This schema standardizes `ucel/coverage/<venue>.yaml` so CI can mechanically prove coverage progress and detect gaps before merge.

## v1 Schema
```yaml
venue: gmocoin
scope: public_private
strict: true
implemented: true
tested: true
entries:
  - id: openapi.public.ws.trades.snapshot
    kind: ws
    access: public
    implemented: true
    tested: true
  - id: openapi.private.rest.order.create
    kind: rest
    access: private
    implemented: true
    tested: true
```

## Field rules
- `venue` (required in v1 final): exchange id, must match file name stem.
- `scope` (required in v1 final): `public_only` or `public_private`.
- `strict` (required): if `true`, CI rejects any incomplete coverage state.
- `implemented`, `tested` (required in v1 final): venue-level summary booleans.
- `entries` (required): list of tracked API units.
  - `id` required, must exist in `docs/exchanges/<venue>/catalog.json`.
  - `kind` enum: `rest` | `ws`.
  - `access` enum: `public` | `private`.
  - `implemented`/`tested` are endpoint-level booleans.
  - `notes` optional for temporary exceptions.

## Strict policy
`strict: true` means:
- venue-level `implemented` and `tested` must both be `true`.
- every `entries[]` item must have `implemented: true` and `tested: true`.

## Scope policy
- Domestic venues: `public_private` by default.
- Global venues: `public_only` by default.
- Exceptions must be documented in `entries[].notes` and verification notes.

## Rollout / versioning
- Current gate keeps backward compatibility by accepting missing new fields during migration.
- Once all coverage files carry v1 fields, gate will switch from optional to required enforcement.
- v2 must preserve v1 keys or provide explicit migration notes and compatibility checks.
