# UCEL-TRANSPORT-STABILITY-004 Verification

## 1) Changed files
- docs/runbooks/transport/ws_overflow_spill.md
- docs/runbooks/transport/ws_rate_limit_cooldown.md
- docs/runbooks/transport/ws_reconnect_storm.md
- docs/specs/crosscut/support_bundle_spec.md
- docs/specs/market_data/ws_transport_stability_spec.md
- docs/status/trace-index.json
- docs/verification/UCEL-TRANSPORT-STABILITY-004.md

## 2) What / Why
Completed docs SSOT alignment for WS transport stability operations by upgrading the market-data stability spec and three transport runbooks to operational-ready versions. Added explicit support-bundle transport key requirements and security/redaction constraints in crosscut spec. Updated trace-index task metadata for this task and linked verification evidence.

## 3) Self-check
- Allowed-path check: only `docs/**` touched.
- JSON validity:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- docs link existence check (only `docs/` refs in touched docs):
  - custom python checker => PASS
- Secrets scan:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" docs/runbooks/transport docs/specs/market_data/ws_transport_stability_spec.md docs/specs/crosscut/support_bundle_spec.md` reviewed; no secret values added.
