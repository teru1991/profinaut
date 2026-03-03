# Redaction & No-Leak Contract (Domain B / Step1)

## Purpose
This document is the SSOT contract for preventing secrets / near-secrets from leaking into any output surface:
- logs / traces / metrics labels
- audit.jsonl
- HTTP responses (including error detail)
- support bundles / diagnostics

This SSOT is intentionally conservative ("mask widely") to guarantee safety by default.

## Definitions
### Secret
Any of:
- tokens / api keys / passwords / private keys (PEM) / Authorization headers
- JWT (three-part base64url tokens)

### Near-secret
Any suspicious identifier that can be operationally sensitive:
- long hex/base64 blobs, address-like ids, account id markers, whitelist markers
- policy may later distinguish if allowed for internal-only surfaces, but this base layer must detect them.

## Contract
1) `libs.safety_core.redaction.redact(obj)` MUST:
- recursively mask any key whose name indicates secrets (token/secret/api_key/password/authorization...)
- recursively mask content patterns (Bearer/Basic, JWT, PEM, token=..., long hex/base64)

2) `scan_text(text)` / `scan_obj(obj)` MUST:
- detect secrets and near-secrets even if they appear inside larger strings
- report findings as `RedactionFinding(kind, severity, context_key)`

3) `JsonlAuditWriter.write_event(event)` MUST:
- apply redaction to entire payload (safe default)
- fail-closed (raise `AuditLeakError`) if `scan_obj(payload)` finds any secret indicators after redaction
- NEVER include raw sensitive values in exception messages

## Non-goals (for Step1)
- policy-based allow/deny per surface (will be added in B-STEP2..)
- audit health propagation into dangerous-ops gating (will be added in later steps)

## Test evidence
See: `tests/test_redaction_no_leak.py`
