# UCEL T/Y 100% Definition Spec v1

> LEGACY NOTICE
> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
> Current SSOT is coverage_v2 under `ucel/coverage/coverage_v2/` and `ucel/docs/policies/coverage_policy.md`.

## Scope
- **T**: Testing/Chaos (`golden` / compat / determinism / fuzz / chaos / ssot-gate).
- **Y**: Supportability (`support_bundle` + redaction + reproducibility).

This document is an additive supplement to existing UCEL SSOT docs. It does not replace existing H/T/Y specs.

## T = 100% when
1. Every `strict: true` venue in `ucel/coverage/*.yaml` has required WebSocket golden files:
   - `ucel/fixtures/golden/ws/<venue>/raw.json`
   - `ucel/fixtures/golden/ws/<venue>/expected.normalized.json`
2. Golden fixtures are manifest-managed (`ucel/fixtures/golden/manifest.json`) with deterministic `sha256` + `bytes` metadata.
3. Golden fixtures pass deny-pattern redaction checks (no credential/token style strings).
4. Oversized fixture payloads are rejected by gate policy (`>1MiB` in manifest gate) to keep deterministic CI bounds.

## Y = 100% when
1. Support bundle output includes the minimum reproducibility keys declared in `ucel/fixtures/support_bundle/manifest.json`.
2. Support bundle output includes the same manifest payload for machine-verifiable compatibility.
3. Support bundle payload is deny-pattern scanned to prevent secrets leakage.
4. Reproduction-critical diagnostics remain present (`health`, `metrics`, `events_tail`, `rules_snapshot`, observability text).

## Acceptance workflow
- Golden manifest refresh: `python3 scripts/ucel/update_golden_manifest.py --accept`.
- Support bundle manifest normalization: `python3 scripts/ucel/update_support_bundle_manifest.py`.
- CI gate is test-driven; out-of-date manifest or missing required fields fails tests.
