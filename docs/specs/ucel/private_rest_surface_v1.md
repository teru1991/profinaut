# UCEL Private REST Surface v1

## Scope
This spec defines the canonical private REST surface for JP-resident policy-aware UCEL integrations.
All private REST calls must pass Policy Gate + Auth Core before any network send.

## Canonical operations
- `get_balances`
- `get_open_orders`
- `get_order`
- `cancel_order`
- `get_fills`
- `get_account_profile`
- `get_positions`

## Operation class
- Read-only private: `get_balances`, `get_open_orders`, `get_order`, `get_fills`, `get_account_profile`, `get_positions`
- Write private: `cancel_order`

## Auth contract
- `requires_auth=true`
- `key_id` is required
- Sign/material resolution must run through Auth Core runtime (`ucel-core::auth` + `ucel-transport::auth`)

## Reason code normalization
Canonical reject classes:
- `unauthorized`
- `forbidden`
- `permission_denied`
- `validation_failed`
- `insufficient_funds`
- `not_found`
- `rate_limited`
- `retryable_transport`
- `permanent_venue_reject`
- `not_supported`

## Retry safety
- Read operations may be retry-safe on `429` / `5xx`.
- Write operations are fail-closed (`unsafe_to_retry`) unless explicit idempotency evidence exists.

## Policy alignment
- Venue scope `public_private` => private REST surface may be enabled.
- Venue scope `public_only` / `blocked` => private REST surface must fail before network send.
