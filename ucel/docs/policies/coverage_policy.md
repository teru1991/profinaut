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

## Discoverability gate
- If `public.ws=true` in coverage_v2 for a venue, `supported_ws_ops()` MUST NOT be empty for that venue implementation.
- CI enforces this via `discoverability_coverage_v2_gate` and alignment checks in `coverage_gate`.
## Strict SSOT
- Strict CI gates use `ucel/coverage/coverage_v2/strict_venues.json` as the only strict venue source.
- `ucel/coverage/*.yaml` (v1) is legacy/informational and MUST NOT be used for CI gating.

## 4. Venue access policy linkage (JP resident)
- Machine-readable venue access SSOT is `ucel/coverage/coverage_v2/jurisdictions/jp_resident_access.json`.
- Default scope is `public_only`; venues without explicit entries are treated as public_only.
- Explicit JP resident `public_private`: `bitbank`, `bitflyer`, `coincheck`, `gmocoin`.
- Explicit JP resident exception: `sbivc` remains `public_only`.
- Hub/Registry/Invoker MUST fail-fast private REST/WS/execution requests when scope disallows private surfaces.
