# Dangerous Operations Confirmation Specification

## 1. Purpose
This document defines a single canonical confirmation contract for dangerous operations across UI/API/Audit.

Goals:
- Require human-readable reason.
- Enforce double-confirm challenge for dangerous operations.
- Enforce confirmation expiry (TTL).
- Guarantee auditable outcomes.
- Enforce capability gating with safe defaults (OFF => no dangerous ops).

## 2. Applicability
This spec applies to all operations classified as dangerous by `docs/specs/dangerous-ops-taxonomy.md`.
- T0/T1: double-confirm flow in this document is REQUIRED.
- T2: implementation MAY use single-confirm, but if double-confirm is used it MUST follow this spec exactly.

## 3. Canonical Request Fields
Dangerous-op request payloads MUST support these canonical fields:

- `reason: string`
  - Required when dangerous-op enforcement is enabled.
  - Must be human-readable free text supplied by actor.
- `confirm_token: string | null`
  - Opaque server-issued token returned at challenge step.
  - `null`/missing on first attempt.
- `confirm_expires_at: string (ISO-8601 UTC) | null`
  - Server-issued value indicating expiration timestamp.
  - Client MUST NOT set authoritative value; server determines validity.
- `confirm_intent_hash: string | null`
  - Optional but recommended.
  - If used, binds confirmation token to normalized operation intent payload.

## 4. Capability Gating and Safe Defaults
1. Dangerous-op capability is **OFF by default**.
2. When capability is OFF:
   - API MUST return `DANGEROUS_OPS_DISABLED`.
   - API MUST NOT call upstream trading/control services.
   - UI MUST hide or disable dangerous action controls.
3. Capability check MUST occur before any side-effecting execution path.

## 5. Two-Step API Flow (Normative)

## Step A: Challenge issue (no execution)
Client sends dangerous-op request with:
- `reason` present
- `confirm_token` absent or `null`

Server behavior:
1. Validate capability ON.
2. Validate `reason` presence and non-empty content.
3. Create confirmation challenge/token and expiry.
4. Return error response `CONFIRMATION_REQUIRED` including `confirm_token` and `confirm_expires_at`.
5. **No side effects on Step A**:
   - MUST NOT place/replace/cancel orders.
   - MUST NOT perform kill-switch state transition.
   - MUST NOT call upstream side-effecting endpoints.

Recommended HTTP status: `409 Conflict`.

## Step B: Confirmation execution
Client re-sends the **same intended operation** with:
- Same dangerous-op payload fields
- `reason` present
- `confirm_token` set to issued token
- (Optional) same `confirm_intent_hash` when used

Server behavior:
1. Validate capability ON.
2. Validate token exists and not expired (`now <= confirm_expires_at`).
3. Validate token-intent match.
4. If valid, execute operation exactly once (respecting existing idempotency controls).
5. Return normal success response.

Recommended HTTP status: `200 OK` / operation-specific success code.

## 6. Error Model (Canonical Codes)
The following codes are mandatory for dangerous-op enforcement:

- `REASON_REQUIRED`
  - Trigger: `reason` missing/empty when enforcement enabled.
  - Recommended status: `400 Bad Request`.
- `CONFIRMATION_REQUIRED`
  - Trigger: Step A request without `confirm_token`.
  - Response MUST include `confirm_token`, `confirm_expires_at`.
  - Recommended status: `409 Conflict`.
- `CONFIRMATION_EXPIRED`
  - Trigger: token exists but now > expiry.
  - Recommended status: `409 Conflict`.
- `CONFIRMATION_MISMATCH`
  - Trigger: token does not match operation intent/payload or actor/context.
  - Recommended status: `409 Conflict`.
- `DANGEROUS_OPS_DISABLED`
  - Trigger: capability OFF.
  - API MUST reject before upstream calls.
  - Recommended status: `403 Forbidden`.

Error response shape (minimum):

```json
{
  "error": {
    "code": "CONFIRMATION_REQUIRED",
    "message": "confirmation token required",
    "confirm_token": "...",
    "confirm_expires_at": "2026-02-15T12:34:56Z"
  }
}
```

For non-confirmation errors, `confirm_token` and `confirm_expires_at` MAY be omitted or null.

## 7. Audit Requirements (Normative)
Server MUST emit these events for dangerous operations:
- `dangerous_op_challenge_issued`
- `dangerous_op_confirmed`
- `dangerous_op_rejected`

Each event MUST include fields:
- `ts_utc`
- `service`
- `event`
- `request_id`
- `actor`
- `op_name`
- `reason`
- `expires_at`
- `result`

Additional guidance:
- `result` should be explicit (`issued` / `confirmed` / `rejected:<code>`).
- Rejections from capability OFF, reason missing, expiry, mismatch all emit `dangerous_op_rejected`.

## 8. Intent Matching Rules
To prevent token replay/misuse:
1. Token MUST be bound to actor identity and operation name.
2. Token SHOULD be bound to canonicalized payload hash (`confirm_intent_hash`).
3. Any mismatch MUST return `CONFIRMATION_MISMATCH`.

## 9. TTL / Expiry Rules
1. Server issues `confirm_expires_at` for every challenge.
2. Confirmation is valid only while `now <= confirm_expires_at`.
3. Expired challenges MUST return `CONFIRMATION_EXPIRED` and require a new Step A.
4. Clients should treat tokens as single-use; server SHOULD invalidate after successful Step B.

## 10. UI Behavior Requirements
1. UI must require reason entry before initial submit.
2. On `CONFIRMATION_REQUIRED`, UI must present explicit second confirmation step and expiry countdown.
3. UI must re-submit same intent with `confirm_token`.
4. If capability OFF, UI must hide/disable controls and surface disabled rationale.

## 11. Degraded/Failure Behavior
- If confirmation subsystem unavailable, server MUST fail closed for dangerous operations (reject; no upstream side effects).
- No dangerous op may proceed without satisfying reason + confirmation requirements when enforcement is enabled.
