# Observability Data Classification (SSOT)

This document is the SSOT for what may be emitted through Observability surfaces:

- structured logs
- `/healthz` and `/capabilities` payloads
- diagnostics / support bundles

## Classes

### Public

Information safe to expose broadly. Public fields MUST be explicitly designated. Default is not Public.

### Internal

Operationally useful data for trusted system boundaries. Internal data should still be minimized.

### Restricted (MUST NOT EMIT RAW)

Any data that can lead to compromise, privacy leakage, or trading incidents. Restricted data MUST be masked (`"***"`) or dropped, and never emitted raw.

Examples (non-exhaustive):

- Secrets / credentials / signature materials:
  - `authorization`, `cookie`, `token`, `api_key`, `secret`, `password`, `signature`, `nonce`
- Trade/order details:
  - `client_order_id`, `order_id`, `price`, `qty`, `size`, `amount`, `notional`
- Infrastructure-sensitive:
  - `host`, `hostname`, `ip`, `internal_url`, `base_url`, `endpoint`
- Personal info:
  - `email`, `phone`, `address`

## Enforcement

- Runtime: deep redaction MUST run before emitting logs or returning health/capabilities payloads.
- CI: redaction lint MUST fail pull requests that introduce raw secret-like output patterns in logging paths.

## References

- `docs/policy/forbidden_keys.toml`
- `docs/policy/redaction.toml`
