# Safety Kill Runbook (UI + Local)

## Purpose
This runbook defines dual-path kill operations for Safety Controller Domain E:
- UI kill via API endpoint (`POST /safety/kill`)
- Local kill via kill-file path (worker-side, no UI/network dependency)

## UI Kill Procedure
1. Send `POST /safety/kill` with required fields:
   - `requested_mode`
   - `scope_kind`
   - `selector`
   - `ttl_seconds` (required, >0)
   - `reason` (required)
   - `actor` (required)
   - `idempotency_key` (required)
   - `evidence` (must include trace/run/audit id)
2. Verify response includes current state and decision summary.
3. For downgrade (`requested_mode=NORMAL`), expect rejection unless all conditions pass:
   - stable period
   - health OK
   - reconcile OK

## Local Kill Procedure (Network/UI Down)
1. Create local kill trigger file: `/var/run/profinaut_kill_switch.json`
2. Include secret-free payload fields (IDs only):
   - `reason`
   - `trace_id`
   - `audit_id`
3. Run worker local kill runner (`worker/local_kill_runner.py` path).
4. Confirm state is `EMERGENCY_STOP` and audit trail exists (or deferred write if audit backend unavailable).

## Release/Downgrade Conditions (Hard Gate)
Never release from `EMERGENCY_STOP`/`SAFE` unless all are satisfied:
- stability window satisfied (default 5min)
- health checks pass
- reconcile checks pass

## Support Bundle Collection (Secret-Zero)
Use support bundle collection from `libs/safety_core/support_bundle.py` to export:
- `safety_state.json`
- `active_directives.json`
- `recent_audit.jsonl` (redacted)
- `health_snapshot.json`
- `config_hash.txt`

## Forbidden Practices
- Never issue exception/suppression without TTL.
- Never include secrets/PII in reason/evidence/support bundle.
- Never assume UI path availability; always maintain local kill readiness.
