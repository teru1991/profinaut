# Security Hardening Threat/Incident Spec (SSOT)

## Goals
- **No secret leakage**: secrets must never appear in logs, errors, support bundles.
- **Misconfiguration-proof**: endpoints must be validated (https/wss only + allowlist).
- **DoS-resistant**: enforce input limits (max frame bytes, max JSON depth/bytes) *before* parsing.

## Mandatory rules
1) Redaction-before-output
- Any outbound log that includes headers/query/body MUST be redacted via `ucel_transport::security::redaction`.
- Sensitive keys are masked (case-insensitive): authorization, api-key, secret, signature, passphrase, cookie, token.

2) Endpoint validation
- Only `https://` and `wss://` are allowed.
- Host must be in allowlist (exact match; optional subdomain policy).
- Failure must be a hard error at configuration/initialization time.

3) JSON limits
- Prior to JSON parse, enforce:
  - max bytes
  - max depth (object/array nesting)
- Violation returns a non-retryable UCEL error.

(Integration into ws/http/registry/adapters happens in Step2/Step3.)
