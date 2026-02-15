# Dangerous Operations Taxonomy

## 1. Purpose
This taxonomy defines which operations are considered **dangerous operations** in Profinaut and the minimum UX protections required before execution.

The taxonomy is normative for UI/API/audit implementations.

## 2. Scope and Terms
- **Dangerous operation**: Any operation that can immediately create market exposure, materially change risk posture, or affect production safety controls.
- **LIVE mode**: Production trading mode where real orders and account balances are impacted.
- **Capability gate**: Service/UI feature switch indicating dangerous ops are enabled. Default is OFF.

## 3. Risk Tiers

### T0 — Critical market-impacting actions
Operations with immediate and potentially irreversible LIVE impact.

Examples:
- Enable LIVE trading.
- Disable LIVE trading when it could bypass expected kill-switch pathways.
- Place order in LIVE.
- Replace/amend order in LIVE.
- Kill-switch actions: `CLOSE_ONLY`, `FLATTEN`, `HALT`.
- `cancel-all` in LIVE.
- Bulk order actions in LIVE (bulk place/replace/cancel).

Required UX/API protections:
1. Capability gate must be ON.
2. Reason input is required.
3. Double-confirm challenge is required.
4. Challenge has TTL (`confirm_expires_at`); expired tokens are rejected.
5. Confirmation must bind to the same intent payload (hash or equivalent matching).
6. Full audit events for issued challenge + confirmed/rejected execution.

### T1 — High operational control actions
Operations that may not directly place exposure but can alter live safety posture or production behavior.

Examples:
- Force reconcile execution/portfolio state.
- Canary promote for production execution path.
- Bulk control-plane actions that alter multiple running bots/services.

Required UX/API protections:
1. Capability gate must be ON.
2. Reason input is required.
3. Double-confirm challenge is required.
4. Challenge TTL enforcement is required.
5. Confirmation must bind to the same intent payload (hash or equivalent matching).
6. Full audit events are required.

### T2 — Elevated but limited-blast operations
Operations with bounded impact that still require traceability and explicit operator intent.

Examples:
- Non-LIVE bulk maintenance actions.
- Administrative resets with no direct LIVE order side effects.

Required UX/API protections:
1. Capability gate must be ON.
2. Reason input is required.
3. Single confirmation MAY be used (implementation choice), but audit is required.
4. If double-confirm is implemented for consistency, use the same canonical confirmation fields.

## 4. Classification Rules
To classify an operation:
1. If it can place/modify/cancel LIVE orders or trigger kill-switch actions, classify as **T0**.
2. Else if it changes production safety or rollout posture with system-wide effect, classify as **T1**.
3. Else if impact is bounded and reversible with no direct LIVE order side effect, classify as **T2**.

If uncertain between tiers, choose the **higher-risk tier**.

## 5. Repository Baseline Classification (Normative)
The following operations are dangerous and MUST be treated as at least the listed tier:

| Operation | Minimum Tier |
|---|---|
| LIVE trading enable/disable | T0 |
| Place/replace orders in LIVE | T0 |
| Kill-switch actions (`CLOSE_ONLY` / `FLATTEN` / `HALT`) | T0 |
| Cancel-all / LIVE bulk order actions | T0 |
| Force reconcile | T1 |
| Canary promote | T1 |

## 6. Safe Default Policy
- Capability gate for dangerous ops is **OFF by default**.
- When OFF, dangerous operations are not permitted in any tier.
- UI must hide or disable dangerous actions when capability is OFF.
- API must reject requests before calling upstream dependencies when capability is OFF.

## 7. Non-Ambiguity Requirements
Implementations MUST:
- Use canonical field names and error codes from `docs/specs/dangerous-ops-confirmation.md`.
- Enforce no-side-effect challenge issuance for double-confirm flows.
- Emit required audit fields for all dangerous-op outcomes.
