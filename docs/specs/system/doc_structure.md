# Documentation Structure (SSOT) v1.0
Status: Canonical
Scope: Where each kind of document must live

## 0. Rules (Non-negotiable)
1) Do not multiply SSOTs: one canonical place per purpose.
2) Contracts live only in `docs/contracts/**`.
3) Fixed meaning lives only in `docs/specs/**`.
4) Tunable values live only in `docs/policy/**`.
5) Plans live only in `docs/plans/**`.
6) Procedures live only in `docs/runbooks/**`.
7) Old documents go to `docs/legacy/**` and must not be used as truth.

---

## 1. Canonical Directories
- `docs/contracts/` : JSON Schema contracts (machine-readable)
- `docs/specs/`     : Core fixed specs
  - `docs/specs/ucel/`      : UCEL core specs
  - `docs/specs/crosscut/`  : crosscut core specs (safety/audit/bundle)
  - `docs/specs/system/`    : system-wide SSOT vocabulary/structure/versioning
- `docs/policy/`     : tunable values (toml/markdown)
- `docs/plans/`      : roadmap/milestones
- `docs/runbooks/`   : operational procedures
- `docs/status/`     : runtime status SSOT and trace SSOT
- `docs/handoff/`    : handoff SSOT
- `docs/decisions/`  : decision log SSOT
- `docs/context/`    : operator/AI entry hub (NOT SSOT)
- `docs/legacy/`     : non-canonical references

---

## 2. Stubs
When migrating a doc to legacy, keep a small stub at the original path that:
- states “NOT CANONICAL”
- links to the new canonical doc
- contains no normative content
