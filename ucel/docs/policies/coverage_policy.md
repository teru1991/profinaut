# UCEL Coverage Policy (SSOT)

## 1. Source of Truth
- The source of truth for exchange capability coverage is **coverage_v2** under `ucel/coverage/coverage_v2/`.
- Legacy `ucel/coverage/*.yaml` (v1) is informational only unless explicitly stated otherwise.

## 2. Domestic vs Overseas requirements
### Domestic exchanges (Japan)
Domestic exchanges must support:
- Public REST
- Public WebSocket
- Private connectivity (REST and/or WS depending on venue), expressed as `private.enabled=true` in coverage_v2.

Domestic exchanges in scope:
- gmocoin
- bitbank
- bitflyer
- coincheck

### Exception: SBIVC (sbivc)
- sbivc is **temporarily public_only** due to missing/insufficient private API documentation.
- This exception MUST be enforced by tests: **sbivc is the only domestic exchange allowed to be public_only**.

#### Exit criteria (to remove exception)
- Private API documentation acquired and verified (auth/nonce/clock drift/permissions).
- Private endpoints implemented with deterministic signing tests.
- Coverage updated to `private.enabled=true` and gated in CI.

## 3. Discoverability alignment
Each CEX crate must provide capability discoverability (e.g. `supported_ws_ops()`), and:
- If `public.ws=true` in coverage_v2, the supported WS ops list must be non-empty unless the venue explicitly has no public ws ops.
- Any mismatch must fail in CI.

## Legacy note
- coverage v1 is legacy and not used for CI gating.
