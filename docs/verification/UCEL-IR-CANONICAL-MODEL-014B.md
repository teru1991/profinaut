# UCEL-IR-CANONICAL-MODEL-014B Verification

## 1) Changed files (`git diff --name-only` target)
- docs/specs/ucel/ir_canonical_surface_v1.md
- docs/specs/ucel/ir_fetch_contract_v1.md
- docs/specs/ucel/ir_identity_contract_v1.md
- docs/specs/ucel/ir_artifact_contract_v1.md
- ucel/docs/ir/ir_canonical_model.md
- ucel/docs/ir/ir_fetch_flow.md
- ucel/docs/ir/ir_access_guard_policy.md
- ucel/Cargo.lock
- ucel/crates/ucel-core/src/ir.rs
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-ir/Cargo.toml
- ucel/crates/ucel-ir/src/lib.rs
- ucel/crates/ucel-ir/src/model.rs
- ucel/crates/ucel-ir/src/identity.rs
- ucel/crates/ucel-ir/src/document.rs
- ucel/crates/ucel-ir/src/artifact.rs
- ucel/crates/ucel-ir/src/fetch.rs
- ucel/crates/ucel-ir/src/access.rs
- ucel/crates/ucel-ir/src/errors.rs
- ucel/crates/ucel-sdk/Cargo.toml
- ucel/crates/ucel-sdk/src/ir.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/src/ir_canonical.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/ir_canonical_model_gate.rs
- ucel/crates/ucel-testkit/tests/ir_fetch_contract_gate.rs
- ucel/crates/ucel-testkit/tests/ir_identity_contract_gate.rs
- ucel/crates/ucel-testkit/tests/ir_docs_drift_canonical.rs
- ucel/examples/ir_canonical_preview.rs
- ucel/fixtures/ir_canonical/source_descriptor_fixture.json
- docs/status/trace-index.json
- docs/verification/UCEL-IR-CANONICAL-MODEL-014B.md

## 2) What/Why
This task fixes the canonical IR contracts before source-specific expansion in 014C-014F. We added typed IR models in `ucel-core`, canonical fetch/identity/document/artifact/access contracts in `ucel-ir`, and SDK/registry/testkit surfaces to prove reachability and drift detection. Access-policy decisions are now represented as typed guards with explicit `Allowed/ReviewRequired/Blocked`. The contract supports API/feed/HTML/attachment modes in one trait while preserving source metadata and provenance. We intentionally avoided broad source-adapter expansion and localized changes to model/contracts/docs/gates.

## 3) Self-check results
- Allowed-path check: OK (staged files only, allowlist expression matched empty violations).
- Tests added/updated OK:
  - ir_canonical_model_gate
  - ir_fetch_contract_gate
  - ir_identity_contract_gate
  - ir_docs_drift_canonical
- Build/Unit test command results:
  - `python -m json.tool ucel/coverage_v2/ir/ir_inventory.json > /dev/null` OK
  - `rg -n '"market": "jp"|"market": "us"' ucel/coverage_v2/ir/ir_inventory.json` OK
  - `cd ucel && cargo test -p ucel-core -p ucel-ir -p ucel-registry -p ucel-sdk` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_canonical_model_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_fetch_contract_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_identity_contract_gate -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_docs_drift_canonical -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit` started, but stopped due long-running full testkit compilation/execution in this environment.
- trace-index json.tool OK (`python -m json.tool docs/status/trace-index.json > /dev/null`).
- Secrets scan OK (`rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` over touched files; no credential additions).
- docsリンク存在チェック OK (new docs with `docs/` references resolved).

## 4) ★履歴確認の証拠
- `git log --oneline --decorate -n 50` reviewed latest baseline around `766b669` (014A) and `8ddfdca` (009F), no reverts around IR inventory introduction.
- `git log --graph --oneline --decorate --all -n 80` reviewed branch topology and merge continuity.
- `git show HEAD --stat` reviewed pre-change baseline commit intent.
- last-touch inspection executed for:
  - `ucel/coverage_v2/ir/ir_inventory.json`
  - `ucel/crates/ucel-ir/src/lib.rs`
  - `ucel/crates/ucel-core/src/lib.rs`
  - `ucel/crates/ucel-sdk/src/lib.rs`
  - `ucel/crates/ucel-registry/src/hub/registry.rs`
- blame/log review executed for `ucel-ir`, `ucel-core`, `ucel-sdk/src/lib.rs`, `registry/hub/registry.rs`, `ucel/docs/ir`.
- `git reflog -n 30`, `git branch -vv`, `git log --merges --oneline -n 30` reviewed local movement and merge evidence.
- `git merge-base HEAD origin/master` failed because `origin/master` does not exist in this environment (documented limitation).

### 設計根拠
- Canonical model enums and descriptors are direct codification of 014A taxonomy/policy vocabulary.
- Fetch contract includes API/feed/HTML/attachment parity through `IrFetchMode` + single adapter trait.
- Identity contract requires source-scoped provenance and rejects empty provenance.
- Access guard maps policy classes to `Allowed/ReviewRequired/Blocked` with explicit excluded-class fail-fast.

### 014A inventory から contract へ落とし込んだ根拠
- `source_family/source_kind/access_policy_class/access_patterns` -> `IrSourceDescriptor` model fields.
- `identity_kind` matrix -> `IrIssuerIdentityKind` enum and resolution input/result.
- `document_family/artifact_kind` -> canonical document/artifact enums and descriptor contracts.

### evidence conflict
- No contradiction detected; unresolved source implementations remain intentionally deferred to 014C-014F while contracts are now fixed.
