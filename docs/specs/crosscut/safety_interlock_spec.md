# Crosscut Core Spec: Safety Interlock (Policy A) v1.0
Status: Canonical / Fixed Contract (Core Spec)
Scope: System Safety Mode (contract SSOT) + Execution Kill-Switch (separate concept) + enforcement boundaries

## 0. Purpose (Non-negotiable)
This document defines the single canonical crosscut safety model that:
- prevents catastrophic actions by design (default-safe),
- remains fully consistent with contract SSOT (`docs/contracts/safety_state.schema.json`),
- separates *policy values* (thresholds/timers/sensitivities) into `docs/policy/*`,
- produces auditable, replayable evidence for every state transition.

This spec is **Core fixed**. Operational thresholds MUST live in Policy.

---

## 1. Canonical Concepts

### 1.1 System Safety Mode (SSOT Contract)
**System Safety Mode** is a top-level operating mode of the entire system.
It MUST use the exact enum defined in contract SSOT:

- `NORMAL`
- `SAFE`
- `EMERGENCY_STOP`

Contract reference:
- `docs/contracts/safety_state.schema.json`

This mode is the **only** system-wide safety state machine.

### 1.2 Execution Kill-Switch (Separate Concept)
Execution Kill-Switch is a **domain-level constraint** on trading actions.
It is NOT the same as System Safety Mode (and is intentionally not embedded in `safety_state` schema v1).

Execution Kill-Switch Levels (canonical vocabulary for specs & audit payloads):
- `ALLOW`       : normal trading allowed (subject to usual gates)
- `CLOSE_ONLY`  : only exposure-reducing actions are allowed (reduce-only)
- `FLATTEN`     : actively attempt to neutralize exposure (reduce-only, aggressive flatten intent)
- `BLOCK`       : no execution actions allowed

**Mapping rule (fixed):**
- If System Safety Mode is `EMERGENCY_STOP`, Execution Kill-Switch is effectively `BLOCK`.
- If System Safety Mode is `SAFE`, default is `BLOCK`, but policy MAY allow `CLOSE_ONLY` as an emergency risk-reduction exception (must be audited).
- If System Safety Mode is `NORMAL`, Kill-Switch may still be `CLOSE_ONLY/FLATTEN/BLOCK` due to local hazards (must be audited).

### 1.3 Interlock
An Interlock is a rule that:
- transitions System Safety Mode, and/or
- sets (or tightens) Execution Kill-Switch level, and/or
- rejects specific dangerous operations at enforcement boundaries.

### 1.4 Dangerous Operations (Crosscut Vocabulary)
Dangerous operations include (non-exhaustive):
- execution actions: place/modify/cancel, leverage/margin changes, “start bot live”
- config/apply: plan/config/descriptor apply, coverage gate override
- data ops: delete/evict/restore/compact that may destroy evidence or threaten WAL
- access: relax auth, expose endpoints, bypass safety checks
- overrides: bypass gates, disable monitors, force unlock quarantine

All dangerous operations MUST be:
- explicitly classified,
- challenge-confirmed if required,
- audited.

(See §6 “Dangerous Ops Challenge/Confirmation”.)

---

## 2. Core Principles (Fixed)

1) Default-safe: on uncertainty, choose the safer posture.
2) Observability honesty: missing monitoring makes health unknown → must degrade safety.
3) Evidence-first: safety decisions must be replayable and explainable.
4) No secrets: safety/audit/support bundle must never contain secrets.
5) Separation of concerns:
   - System Safety Mode is contract SSOT and is stable.
   - Execution Kill-Switch is a domain guard and is recorded in audit/details.

---

## 3. Canonical System Safety Modes (Fixed Semantics)

### 3.1 NORMAL
- The system considers itself safe to operate within configured policies.
- Execution is allowed (subject to usual gating & any Kill-Switch constraint).
- Dangerous ops require normal confirmation rules.

### 3.2 SAFE
- Entered when integrity/uncertainty/hazard is detected.
- Default behavior: **block execution** (Kill-Switch defaults to `BLOCK`).
- Policy MAY allow emergency exception `CLOSE_ONLY` to reduce risk.
- Config/apply and destructive data ops are blocked unless explicitly allowed by runbook + audit.

### 3.3 EMERGENCY_STOP
- Entered when catastrophic hazard or integrity loss requires immediate halt.
- Execution is blocked (`BLOCK`).
- Only evidence capture, diagnostics, and controlled recovery actions are allowed.

---

## 4. Interlocks (Inputs and Fixed Semantics)

### 4.1 Gate-driven Interlocks (CI / Runtime / Daily)
Inputs MUST include:
- `gate_results` (contract SSOT)
- `integrity_report` (contract SSOT)

Semantics:
- Runtime gate status `UNKNOWN` (e.g., monitoring down) MUST force System Mode at least `SAFE`.
- Integrity report `FAIL` MUST force System Mode at least `SAFE` and MAY require `EMERGENCY_STOP` depending on hazard category (policy chooses threshold, but *category semantics are fixed*).

### 4.2 Observability Interlock (Honesty Rule)
If observability is degraded (targets down / log ingest down / metrics absent):
- System Mode MUST transition to `SAFE` (minimum).
- An audit event MUST record the missing interval.

### 4.3 Clock / Time Integrity Interlock
If clock drift/rollback is critical:
- System Mode MUST be at least `SAFE`.
- Execution Kill-Switch MUST be `BLOCK` by default (policy may allow `CLOSE_ONLY` exception with audit).
- Config/apply must be blocked (avoid applying changes under time uncertainty).

### 4.4 Persistence / WAL Risk Interlock
If persistence/WAL risk is detected:
- System Mode MUST be at least `SAFE`.
- Block destructive data ops.
- Execution is blocked by default.

### 4.5 Quarantine Interlock
If quarantined streams impact P0 scope above policy threshold:
- System Mode MUST be at least `SAFE`.
- Audit must list quarantined stream identifiers.

### 4.6 Disk/IO Pressure Interlock
If disk near-full or IO stall threatens WAL:
- System Mode MUST be at least `SAFE`.
- Destructive/expensive data ops are blocked unless runbook explicitly permits.

---

## 5. Enforcement Boundaries (Required)

Every domain MUST enforce System Safety Mode + Execution Kill-Switch at boundaries:

### 5.1 Execution Boundary (Pre-trade Gate)
Before any order intent is executed:
- Read System Safety Mode.
- Read Execution Kill-Switch.
- Reject prohibited intents (e.g., new exposure in `CLOSE_ONLY`, all in `BLOCK`).

### 5.2 Bot Control Plane Boundary
Starting/resuming live bots:
- Disallowed in `SAFE` / `EMERGENCY_STOP`.
- Allowed in `NORMAL` only if gates pass and Kill-Switch allows.

### 5.3 Config/Plan Apply Boundary
Applying plan/config/descriptor:
- Disallowed in `SAFE` / `EMERGENCY_STOP` (default).
- Allowed in `NORMAL` if gates pass.

### 5.4 Data Operations Boundary
Delete/evict/restore/compact:
- Must consult System Safety Mode.
- Must fail closed if evidence integrity is at risk.

---

## 6. Dangerous Ops Challenge/Confirmation (Canonical Crosscut Rule)

Dangerous operations MUST be protected by a challenge/confirmation protocol.
This spec makes it crosscut-canonical so it cannot drift.

### 6.1 Required fields for challenge (in audit_event.details)
When a dangerous op is requested, emit `audit_event` with:
- `details.dangerous_op.class` (EXECUTION/CONFIG/DATA/ACCESS/OVERRIDE)
- `details.dangerous_op.intent` (human-readable)
- `details.dangerous_op.challenge_id` (uuid)
- `details.dangerous_op.requires` (e.g., "2-step-confirm", "time-lock-60s")

### 6.2 Confirmation rules (fixed)
- Confirmation MUST be explicit and time-bounded (window is Policy).
- Confirmation MUST include actor identity and reason.
- If the window expires, request MUST be rejected.

### 6.3 System Safety interaction (fixed)
- In `SAFE` / `EMERGENCY_STOP`, confirmations cannot authorize exposure-increasing execution.
- In `EMERGENCY_STOP`, confirmations cannot authorize execution at all.

---

## 7. Required Evidence Artifacts (Contract)

### 7.1 safety_state
System Mode transitions MUST produce a `safety_state` object conforming to contract SSOT.
- `mode`: NORMAL/SAFE/EMERGENCY_STOP
- `reason`: non-empty, must include trigger summary
- `activated_at`: UTC timestamp
Optional:
- `activated_by` (actor)
- `resolved_at`

### 7.2 audit_event
Every transition or dangerous op MUST emit `audit_event` (contract SSOT).
Recommended action naming (convention, not schema constraint):
- `safety.transition`
- `execution.killswitch.set`
- `dangerous_op.challenge`
- `dangerous_op.confirm`
- `dangerous_op.reject`

Recommended resource_type:
- `safety_state`
- `execution_guard`
- `dangerous_op`

Evidence references should be stored in `audit_event.details.refs`:
- `gate_results_ref`
- `integrity_report_ref`
- `startup_report_ref`
- `support_bundle_manifest_ref` (if any)

---

## 8. Policy / Runbook Separation (Hard Boundary)
- Thresholds, timers, “how long stable window”, “what constitutes critical disk” are Policy.
- Operational steps (restore procedures, key rotation, recovery) are Runbooks.
- This doc only defines fixed semantics and invariants.

---

## 9. Versioning
SemVer:
- MAJOR: change meaning of safety modes or mapping rules
- MINOR: add new interlock categories or recommended conventions
- PATCH: editorial clarifications
