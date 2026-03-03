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
