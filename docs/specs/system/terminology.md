# System Terminology (SSOT) v1.0
Status: Canonical (Core Spec)
Scope: Shared vocabulary used across contracts/specs/policy/runbooks

## 0. Purpose
This document defines canonical terms to avoid SSOT drift.
If another document uses conflicting terminology, this doc is the tie-breaker unless an explicit decision overrides it.

---

## 1. SSOT Layers (Canonical)
- Contract SSOT: `docs/contracts/**` (JSON Schema; machine contracts)
- Core Spec (Fixed): `docs/specs/**` (meaning, invariants, boundaries; no tuning constants)
- Policy (Tunable Values): `docs/policy/**` (thresholds, caps, retention; does not change meaning)
- Plan (Changeable): `docs/plans/**` (roadmap, milestones, sequences)
- Runbook (Changeable): `docs/runbooks/**` (operational procedures)

---

## 2. Crosscut
Crosscut specs define system-wide invariants spanning multiple domains.
Canonical location:
- `docs/specs/crosscut/**`

---

## 3. Safety Terms
- System Safety Mode: contract-defined system-wide safety mode (NORMAL/SAFE/EMERGENCY_STOP)
- Execution Kill-Switch: execution-only constraint (ALLOW/CLOSE_ONLY/FLATTEN/BLOCK)
- Interlock: rule that transitions safety mode and/or tightens kill-switch and/or rejects dangerous ops

---

## 4. Evidence Terms
- Audit Event: immutable record of an auditable action (`audit_event` contract)
- Replay Pointers: references to input/output ranges for reproducibility
- Integrity Report: periodic truth statement about data integrity
- Gate Results: outcomes of CI/Runtime/Daily gates

---

## 5. Legacy
Legacy docs are non-canonical references stored under `docs/legacy/**`.
Anything in legacy is not “the truth” unless migrated into SSOT.
