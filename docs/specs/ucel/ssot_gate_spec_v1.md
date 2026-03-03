# UCEL SSOT Gate Spec v1

> LEGACY NOTICE
> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
> Current SSOT is coverage_v2 under `ucel/coverage/coverage_v2/` and `ucel/docs/policies/coverage_policy.md`.

## Gate checks
1. Catalog venue must have corresponding `ucel/coverage/<venue>.yaml`.
2. Coverage `venue` (when present) must match file stem.
3. Coverage `entries[].id` must exist in catalog ids.
4. Scope value (when present) must be `public_only` or `public_private`.
5. If `strict=true`, venue-level and entry-level `implemented/tested` must all be `true`.

## Strict vs non-strict
- `strict=true`: complete implementation/test proof is mandatory.
- `strict=false`: incomplete entries are allowed, but id mismatch is never allowed.

## Rollout compatibility
- Existing legacy coverage files are accepted while new fields are being rolled out.
- Enforcement path: optional-now -> required-after-migration (documented in verification per task).

## Failure message format
- Include `venue`, optional `id`, and concrete reason.
- Examples:
  - `venue=bithumb id=openapi.public.ws.trade.snapshot: coverage id not found in catalog.json`
  - `venue=gmocoin: strict=true requires coverage.tested=true`

## Scope defaults
- Domestic exchanges: `public_private`.
- Overseas exchanges: `public_only`.
- Any exception requires notes in coverage and verification evidence.

## Future extension guard (Q/R domains)
- When new `asset_class`/`venue_kind` dimensions are introduced, they must be additive enums.
- Existing id matching and strict checks must remain stable to avoid SSOT drift.
