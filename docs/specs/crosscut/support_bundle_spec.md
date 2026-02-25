# Crosscut Core Spec: Support Bundle (Manifest-first) v1.0
Status: Canonical / Fixed Contract (Core Spec)
Scope: Secret-free diagnostics bundle with a machine-readable manifest

## 0. Purpose (Non-negotiable)
A Support Bundle is a single diagnostics package that:
- is safe to share (no secrets),
- is verifiable (checksums),
- is referenced from audit,
- contains enough evidence for first-response triage and integrity verification.

This spec is compatible with contract SSOT:
- `docs/contracts/support_bundle_manifest.schema.json`

Retention, triggers, and size caps are Policy/Runbook, not Core.

---

## 1. Primary Artifact: support_bundle_manifest (Contract)
Every bundle MUST include a manifest conforming to:
- `docs/contracts/support_bundle_manifest.schema.json`

A bundle without a valid manifest is invalid.

---

## 2. Invariants (Fixed)

1) **Secret-free**: bundle MUST NOT contain secrets (keys/tokens/headers/cookies/private keys).
2) **Central redaction**: same redaction rules as logs/audit apply.
3) **Integrity-checkable**: manifest contains file paths + sizes; sha256 is strongly recommended.
4) **Evidence-linked**: bundle must reference run identity + evidence artifact refs.
5) **Fail closed**: if forbidden content is detected, generation must:
   - fail, or
   - produce an incomplete bundle with explicit failure status recorded in audit (without secrets).

---

## 3. Required Minimum Contents (Fixed)
This section defines what must be present (as files listed in manifest).

### 3.1 Manifest
- `support_bundle_manifest.json` (or equivalent) must be included and listed in itself.

### 3.2 Run Identity Evidence (Required)
Include at least:
- `startup_report.json` (contract-compatible)

### 3.3 Gate + Integrity Evidence (Required when available)
Include:
- `gate_results.json` (latest relevant window)
- `integrity_report.json` (latest relevant window)

If not available (e.g., early crash), include a short `bundle_notes.txt` describing what is missing and why.

### 3.4 Safety State Evidence (Required)
Include:
- `safety_state.json` (current)
Optionally:
- a short excerpt file summarizing recent transitions (must be secret-free).

---

## 4. Recommended Contents (Strongly Recommended)
- health snapshots:
  - `/healthz` output (redacted)
  - `/metrics` snapshot (redacted; avoid labels that may embed secrets)
- quarantine summary (stream ids, reasons, durations)
- environment summary:
  - OS/kernel, CPU/mem, disk usage, IO pressure summary
- configuration snapshots:
  - only redacted configs, or hashes + pointers (never raw secrets)

---

## 5. Forbidden Content (Fixed)
Bundle MUST NOT include:
- API keys, secrets, tokens, passphrases, private keys
- Authorization headers/cookies
- full `.env` files unless line-by-line redacted
- raw request payloads likely to contain secrets

If forbidden content is detected:
- emit `audit_event` with `action=support_bundle.created` outcome `FAILURE` (or `REJECTED`)
- include only non-sensitive diagnostics about the failure

---

## 6. Audit Integration (Fixed)
When a bundle is generated:
- emit `audit_event` referencing the manifest in `details.refs.support_bundle_manifest_ref`
- record reason in `details.reason` (manual request / gate fail / integrity fail / safety escalation)

---

## 7. Storage / Transport (Policy-bound)
Archive format and storage location are Policy.
However:
- internal paths must be stable,
- manifest must list all included files with sizes,
- sha256 checksums are recommended for verification.

---

## 8. Versioning
SemVer:
- MAJOR: remove required minimum contents or relax secret-free guarantee
- MINOR: add recommended sections or optional fields
- PATCH: editorial clarifications
