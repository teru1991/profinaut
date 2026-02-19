# DOC-DATAPLATFORM-SSOT-002-FREE Progress Update (2026-02-19)

## Summary
- Added Data Platform SSOT documents under `docs/data-platform/**` covering architecture, storage contracts, lakehouse layout, serving stores, DQ, security, and ops runbook.
- Added ADR-0001..0005 to lock technology decisions to fully free OSS stack.
- Explicitly documented non-adoption of MinIO and Redis.
- Aligned `active_task` between `docs/status/status.json` and `docs/handoff/HANDOFF.json`.

## Key decisions captured
- Raw-first Bronze as immutable SSOT and provenance anchor.
- Silver/Gold deterministic transforms with RawRef lineage.
- Serving split: ClickHouse (OLAP), PostgreSQL (OLTP), Valkey (cache).
- Secret non-persistence enforced via denylist + scrub gate.

## Verification
- `python -c "import json; json.load(open('docs/status/status.json'))"`
- `python -c "import json; json.load(open('docs/handoff/HANDOFF.json'))"`
- `python -c "import json; json.load(open('docs/status/trace-index.json'))"`
- `git diff --name-only`

## Notes
- Scope is docs-only; no code/service changes included.
- Locks used: `LOCK:shared-docs`.
