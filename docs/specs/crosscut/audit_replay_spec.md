# Crosscut Core Spec: Audit + Replay (Policy A compatible) v1.0
Status: Canonical / Fixed Contract (Core Spec)
Scope: Immutable audit trail + replay pointers + integrity evidence chain across runs

## 0. Purpose (Non-negotiable)
This spec defines a system-wide evidence chain that enables:
- accountability (“what happened and why”),
- replayability (at least deterministic-by-input evidence replay),
- cross-run traceability by stable identifiers and references.

This spec is designed to be compatible with the existing contract SSOT:
- `docs/contracts/audit_event.schema.json`
- `docs/contracts/replay_pointers.schema.json`
- `docs/contracts/integrity_report.schema.json`
- `docs/contracts/gate_results.schema.json`
- `docs/contracts/startup_report.schema.json`

No secrets may be recorded.

---

## 1. Contract Artifacts and Their Roles

### 1.1 startup_report (run identity)
Each run MUST emit `startup_report` with:
- binary identity (hash/version)
- SSOT hash / plan hash (if applicable)
- schema versions

### 1.2 audit_event (append-only)
All auditable actions MUST be emitted as `audit_event`.
Properties are contract-fixed; this spec defines conventions (naming + minimal details payload).

### 1.3 replay_pointers (data references)
Replay pointers identify:
- input dataset ranges / partitions / WAL segments
- config snapshots (SSOT/plan/config) by content-addressed id or stable key
- outputs needed for verification

### 1.4 gate_results + integrity_report
- gate_results: PASS/WARN/FAIL outcomes of CI/Runtime/Daily checks
- integrity_report: periodic truth statement about ingestion/persistence/integrity status

---

## 2. Invariants (Fixed)

1) Every run MUST be traceable:
   - `startup_report` exists and is referenced from audit.
2) Every safety transition and dangerous operation MUST be audited.
3) Replay pointers MUST be sufficient to locate:
   - exact inputs used (or evidence boundaries),
   - exact config snapshot identifiers,
   - outputs and integrity evidence.
4) Missing observability MUST be explicitly recorded and reflected in integrity outcomes.
5) Secret-free guarantee:
   - no API keys, tokens, headers, cookies, private keys.

---

## 3. Determinism Model (Fixed Semantics)

External sources are non-deterministic (network timing), so we define determinism as:

### 3.1 Deterministic-by-Input (minimum contract)
Given:
- identical input ranges (replay pointers),
- identical config snapshots (SSOT/plan/config by hash),
- identical binary hash,
then the system must reproduce:
- identical integrity/gate outcomes OR
- produce structured divergence explanation in `audit_event.details`.

Minimum required replay type:
- **Type B: Deterministic evidence replay** (same detection and summaries).
Type A (bit-identical state replay) may be introduced later.

---

## 4. Audit Naming Conventions (Conventions, not schema constraints)

Because `audit_event.schema.json` is generic, this spec defines **recommended** conventions:

### 4.1 action naming (recommended)
- `run.start`, `run.end`
- `safety.transition`
- `execution.killswitch.set`
- `gate.record` (CI/Runtime/Daily)
- `integrity.record`
- `quarantine.enter`, `quarantine.exit`
- `dangerous_op.challenge`, `dangerous_op.confirm`, `dangerous_op.reject`
- `support_bundle.created`

### 4.2 resource_type (recommended)
- `run`, `safety_state`, `execution_guard`, `gate_results`, `integrity_report`, `replay_pointers`, `support_bundle`

### 4.3 details payload (recommended minimal keys)
- `details.refs.*` for evidence references (keys, hashes, object paths)
- `details.reason` (human readable)
- `details.window` (start/end UTC timestamps if periodic)
- `details.diff` (optional: what changed)
- `details.policy_snapshot_ref` (optional: which policy revision applies)

---

## 5. Required Event Coverage (Fixed)

At minimum, the audit stream MUST include:

- RUN_START: references startup_report
- SAFETY_TRANSITION: from/to mode, triggers, evidence refs
- EXECUTION_KILLSWITCH_SET: level, reason, evidence refs
- GATE_RECORD: gate results ref, time window
- INTEGRITY_RECORD: integrity report ref
- QUARANTINE_ENTER/EXIT: affected streams, reason
- SUPPORT_BUNDLE_CREATED: manifest ref
- RUN_END: summary + refs

These are “must exist as events”, but the exact `action` string follows conventions above.

---

## 6. Evidence Linking Rules (Fixed)

Each relevant audit event MUST include stable references in `details.refs`, e.g.:
- `startup_report_ref`
- `gate_results_ref`
- `integrity_report_ref`
- `replay_pointers_ref`
- `support_bundle_manifest_ref`
- `build_binary_hash`
- `ssot_hash`
- `plan_hash`

References may be:
- content-addressed IDs (hashes),
- object store keys,
- filesystem paths,
but must be stable and resolvable in the operational environment.

---

## 7. Time Discipline (Fixed)

- All recorded times MUST be UTC.
- Clock anomalies must be recorded as audit events.
- During clock anomaly windows, integrity must mark results conservatively (unknown/degraded).

---

## 8. Security (Fixed)

- Redaction is mandatory.
- Forbidden-key detection must emit an audit event indicating:
  - what guard triggered,
  - what action was taken,
  - without including the secret content.

---

## 9. Versioning
SemVer:
- MAJOR: change determinism meaning or required evidence semantics
- MINOR: add new recommended conventions or optional fields
- PATCH: editorial clarifications
