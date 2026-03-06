# UCEL Auth Core v1

## Purpose
UCEL Auth Core standardizes private auth prerequisites across private REST, private WS, and execution surfaces.
This contract centralizes secret resolution boundaries, nonce/idempotency/clock handling, and redacted diagnostics.

## Auth Core responsibilities
- Resolve auth material through an explicit secret resolver boundary.
- Build a signer-ready canonical context (`SignContext`) with timestamp, nonce, and idempotency.
- Enforce fail-fast checks before network send when `requires_auth=true` and material is incomplete.
- Provide redacted preview/log surfaces that do not expose secret raw values.

## Core enums
- `AuthMode`: `none`, `hmac_header`, `hmac_query`, `jwt_bearer`, `session_token`, `custom`
- `AuthSurface`: `private_rest`, `private_ws`, `execution`

## Secret slots
- `key_id`
- `api_key`
- `api_secret`
- `passphrase`
- `session_token`
- `refresh_token`

## Clock / nonce / idempotency rules
- Nonce is monotonic per `(venue, key_id, surface)` scope.
- Server time offset updates require monotonic observation timestamps.
- Excessive clock skew beyond configured threshold is a hard failure.
- Execution/private write intents may derive idempotency keys from shared context.

## Redaction rules
Redaction applies to headers/query/body/sign previews and diagnostics.
Sensitive keys (e.g. authorization/api-key/signature/secret/token/passphrase) must be replaced with a stable placeholder.

## Fail-closed requirements
- `requires_auth=true` with missing `key_id` => fail-fast.
- Missing required auth material per auth mode => fail-fast.
- Invalid auth mode (`none` with `requires_auth=true`) => fail-fast.
- Nonce rollback or non-monotonic clock observation => fail-fast.
