# UCEL-IR-INVENTORY-014A Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/ir_inventory_v1.md
- docs/specs/ucel/ir_source_taxonomy_v1.md
- docs/specs/ucel/ir_document_taxonomy_v1.md
- docs/specs/ucel/ir_issuer_identity_v1.md
- docs/specs/ucel/ir_access_policy_v1.md
- ucel/docs/ir/ir_source_scope_policy.md
- ucel/docs/ir/jp_us_ir_source_matrix.md
- ucel/docs/ir/ir_document_taxonomy.md
- ucel/docs/ir/ir_identity_mapping_matrix.md
- ucel/docs/ir/ir_access_policy.md
- ucel/coverage_v2/ir/ir_inventory.schema.json
- ucel/coverage_v2/ir/ir_inventory.json
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/src/ir_inventory.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/ir_inventory_gate.rs
- ucel/crates/ucel-testkit/tests/ir_taxonomy_gate.rs
- ucel/crates/ucel-testkit/tests/ir_access_policy_gate.rs
- ucel/crates/ucel-testkit/tests/ir_docs_drift.rs
- ucel/examples/ir_inventory_preview.rs
- docs/status/trace-index.json
- docs/verification/UCEL-IR-INVENTORY-014A.md

## 2) What/Why
This task creates a machine-readable IR source inventory SSOT for JP/US scope using repository evidence only. The inventory includes source taxonomy, identity taxonomy, document/artifact taxonomy, and access policy classes so follow-up implementation tasks can be complete and non-ambiguous. We intentionally include statutory, timely, issuer-site, API/feed/HTML/attachment classes and mark current status explicitly. New testkit gates verify inventory completeness, taxonomy constraints, access-policy correctness, and docs drift. Registry helper functions were added in read-only form to expose IR inventory data for gate use without changing runtime behavior.

## 3) Self-check results
- Allowed-path check: OK (checked on staged files only).
- Tests added/updated: 
  - ir_inventory_gate
  - ir_taxonomy_gate
  - ir_access_policy_gate
  - ir_docs_drift
- Build/Unit test command results:
  - `python -m json.tool ucel/coverage_v2/ir/ir_inventory.schema.json > /dev/null` OK
  - `python -m json.tool ucel/coverage_v2/ir/ir_inventory.json > /dev/null` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_inventory_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_taxonomy_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_access_policy_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_docs_drift -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-registry -p ucel-testkit` started but was not completed in this environment due long-running full package compile/test runtime.
- trace-index json.tool: OK (`python -m json.tool docs/status/trace-index.json > /dev/null`).
- Secrets scan: OK (`rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` on touched files; no credential-like additions).
- docsリンク存在チェック: OK (new docs that reference `docs/` paths point to existing files).

## 4) ★履歴確認の証拠
- `git log --oneline --decorate -n 50` reviewed around `8ddfdca` and prior 009A-009E series commits; no revert/backout around domestic series baseline.
- `git log --graph --oneline --decorate --all -n 80` reviewed merge topology and branch continuity.
- `git show HEAD --stat` reviewed baseline tip before this task.
- last-touch inspection executed for `ucel/Cargo.toml`, `ucel/crates/ucel-registry/src/hub/mod.rs`, `ucel/crates/ucel-registry/src/hub/registry.rs`, `ucel/crates/ucel-ir/src/lib.rs`, `ucel/crates/ucel-ir/README.md`.
- `git blame -w` reviewed for `ucel/Cargo.toml`, `ucel/crates/ucel-registry/src/hub/mod.rs`, `ucel/crates/ucel-registry/src/hub/registry.rs`.
- `git log --oneline -n 20 -- ucel/crates/ucel-ir|ucel/coverage|ucel/coverage_v2|ucel/docs|ucel/docs/ir` reviewed to determine IR evidence origin and hotspot scope.
- `git reflog -n 30`, `git branch -vv`, `git log --merges --oneline -n 30` reviewed for local movement and merge context.
- `git merge-base HEAD origin/master` failed because `origin/master` ref does not exist in this environment; recorded as environment limitation.

### JP/US IR scope判定根拠
- JP statutory + US SEC official sources are evidenced by `ucel-ir` provider implementations (`edinet`, `sec_edgar`).
- JP timely disclosure (`tdnet`) and issuer-site HTML/PDF/feed source classes are evidenced in connector spec source_type/access pattern descriptions.
- US issuer-site HTML/PDF/feed classes are likewise evidenced in connector spec non-regulatory source abstraction.

### taxonomy / access policy 棚卸し根拠
- Identity kinds use `CanonicalEntityId::{EdinetCode,Cik}` and ticker↔CIK mapping evidence plus connector spec issuer_id normalization requirements.
- Document/artifact classes were fixed using provider artifact abstractions + connector spec document/event scope.
- Access policy classes were explicitly attached to every source/document entry; excluded classes are reserved but not used as implementation targets in this task.

### evidence conflict
- No direct conflict found; however issuer-site/timely classes are spec-evidenced and intentionally marked `not_implemented`/`partial` until later series tasks.
