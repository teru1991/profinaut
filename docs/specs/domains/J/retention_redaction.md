# Retention & Redaction (SSOT)

## Redaction
- Never store secrets (API keys, tokens, signatures, seed phrases).
- If a value may contain secrets, store only:
  - hash(value) or redacted prefix/suffix (e.g. first 4 chars + "***" + last 2 chars)
- Audit/event logs must redact:
  - Authorization headers
  - query parameters containing signatures/nonces
  - request bodies containing credentials

## Retention
- Audit chain records: 365 days (minimum)
- Policy decisions (decision records): 180 days
- Self-check results: 90 days
- Exceptions (break-glass/suppress): 365 days (minimum)

## Safety rule
- If redaction cannot be guaranteed for an output artifact, DO NOT emit it (fail-close for generation).
