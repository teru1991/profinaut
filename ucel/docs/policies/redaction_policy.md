# UCEL Redaction Policy (Auth Core)

## Scope
This policy covers transport-level redaction for private auth material and signing diagnostics.

## Required redaction targets
- Headers: `Authorization`, `X-API-KEY`, `api-key`, `signature`, token-like keys.
- Query params: signature/token/passphrase/secret-like keys.
- Body fields: secret/token/passphrase/signature-like keys.
- Sign previews and diagnostic strings containing secret-like fragments.

## Behavioral rules
- Key matching is case-insensitive.
- Redacted values use a stable placeholder (`***redacted***`).
- Logging, error messages, and preview surfaces must not contain raw secret values.
- If uncertain whether a field is sensitive, redact it.
