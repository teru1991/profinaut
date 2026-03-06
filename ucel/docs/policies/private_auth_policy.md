# UCEL Private Auth Policy (Domestic Exchanges)

This policy defines UCEL's **SSOT** for private authentication to prevent accidents.

## 1) Signing inputs
- All signing functions MUST be pure: `inputs -> signature`.
- Inputs MUST be explicit:
  - timestamp (string, fixed unit per venue)
  - method (GET/POST/...)
  - path_with_query (exact, no re-order unless venue requires)
  - body (exact string; empty string if none)

## 2) Time drift (server time offset)
- UCEL maintains `ServerTimeOffset` (server_now - local_now).
- Local clock moving backwards MUST be rejected (fail fast).
- Timestamp passed to signers MUST be derived from `now_server(local_now)`.

## 3) Nonce/Sequence
- Use a monotonic counter to avoid collisions across threads.
- Venue-specific formatting can wrap the counter.

## 4) Idempotency & Retry
- PlaceOrder/Cancel/Amend are NOT retryable unless idempotency is enabled for that request.
- 429 MUST apply cooldown.
- 4xx auth/validation are non-retryable.

## 5) Golden gate
- Each domestic venue MUST have golden signing tests with dummy keys.
- Any change in signature output MUST fail CI unless explicitly updated with rationale.

## 6) Coverage policy linkage
- coverage v1 is legacy/informational only.
- CI gating MUST use coverage_v2 and policy gates outside v1 references.
- Domestic venues MUST have mock request-shape gates (wiremock/httpmock) validating method/path/query/body/required headers without real keys.


## 7) Auth Core boundary (UCEL-AUTH-CORE-004)
- Private REST / WS / execution auth preparation MUST pass through shared Auth Core runtime APIs.
- `requires_auth=true` with missing `key_id` or missing mode-specific material MUST fail before transport send.
- Nonce generation MUST be scoped by `(venue,key_id,surface)` and monotonic within scope.
- Sign preview/diagnostic output MUST be redacted per `../policies/redaction_policy.md`.


## 8) Private REST baseline (UCEL-PRIVATE-REST-005)
- Private REST canonical operations MUST map to `docs/specs/ucel/private_rest_surface_v1.md`.
- Private REST for `public_only`/`blocked` venues MUST fail before network send (policy gate first).
- Reject classes and retry-safety MUST be normalized at UCEL layer (no raw venue-only error propagation).
- Contract matrix is tracked in `ucel/../exchanges/private_rest_matrix.md`.

## 9) Private WS baseline (UCEL-PRIVATE-WS-006)
- Private WS login/auth frame generation MUST pass Auth Core runtime.
- Private WS MUST classify ACK model per venue as `explicit_ack` or `implicit_observation`.
- `public_only` / `blocked` venues MUST be rejected before network send.
- Session expiry / reauth / resubscribe MUST be normalized into canonical lifecycle/reject classes.
- Preview/diagnostics/fixtures MUST NOT include raw token/signature/secret/private payload.
