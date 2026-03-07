# UCEL-IR-US-OFFICIAL-014D Verification

## 1) Changed files
- docs/specs/ucel/ir_us_official_surface_v1.md
- docs/specs/ucel/ir_us_official_fetch_policy_v1.md
- docs/specs/ucel/ir_us_official_identity_mapping_v1.md
- docs/specs/ucel/ir_us_official_artifact_policy_v1.md
- docs/specs/ucel/ir_us_official_filing_family_mapping_v1.md
- ucel/docs/ir/us_official_source_matrix.md
- ucel/docs/ir/us_official_fetch_flow.md
- ucel/docs/ir/us_official_identity_mapping.md
- ucel/docs/ir/us_official_artifact_matrix.md
- ucel/docs/ir/us_official_filing_family_mapping.md
- ucel/docs/ir/us_official_access_and_politeness.md
- ucel/crates/ucel-ir/src/lib.rs
- ucel/crates/ucel-ir/src/us_official/mod.rs
- ucel/crates/ucel-ir/src/us_official/sec.rs
- ucel/crates/ucel-ir/src/us_official/identity.rs
- ucel/crates/ucel-ir/src/us_official/document.rs
- ucel/crates/ucel-ir/src/us_official/artifact.rs
- ucel/crates/ucel-ir/src/us_official/fetch.rs
- ucel/crates/ucel-ir/src/us_official/html.rs
- ucel/crates/ucel-ir/src/us_official/feed.rs
- ucel/crates/ucel-ir/src/us_official/download.rs
- ucel/crates/ucel-ir/src/us_official/access.rs
- ucel/crates/ucel-ir/src/us_official/errors.rs
- ucel/crates/ucel-sdk/src/ir.rs
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/src/ir_us_official.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_identity_resolution.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_document_discovery.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_artifact_download.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_access_guard.rs
- ucel/crates/ucel-testkit/tests/ir_us_official_docs_drift.rs
- ucel/examples/ir_us_official_preview.rs
- ucel/fixtures/ir_us_official/issuers.json
- ucel/fixtures/ir_us_official/documents.json
- ucel/fixtures/ir_us_official/artifacts.json
- ucel/fixtures/ir_us_official/artifact_bytes.json
- docs/status/trace-index.json
- docs/verification/UCEL-IR-US-OFFICIAL-014D.md

## 2) What/Why
This task implements the US official IR adapter surface for SEC disclosure sources under the 014A inventory and 014B canonical contracts. The source-specific code is isolated in `ucel-ir/src/us_official/*` and normalizes issuer/document/artifact outputs to canonical UCEL IR types. JSON/API-first with HTML fallback and attachment download policy is formalized in both docs and adapter metadata paths. Access and politeness guard behavior is fixed in code (allowed/review/blocked, user-agent discipline, retry/backoff, and size caps). SDK, registry helpers, fixtures, and gates were added so CI can detect route gaps, mapping drift, and docs/code mismatches.

## 3) Self-check results
- Allowed-path check: OK (staged files only; no allowlist violation in this task patch).
- Tests added/updated:
  - ir_us_official_contract_matrix
  - ir_us_official_identity_resolution
  - ir_us_official_document_discovery
  - ir_us_official_artifact_download
  - ir_us_official_access_guard
  - ir_us_official_docs_drift
- Build/Unit test command results:
  - `python -m json.tool ucel/coverage_v2/ir/ir_inventory.json > /dev/null` OK
  - `rg -n '"market": "us"' ucel/coverage_v2/ir/ir_inventory.json` OK
  - `cd ucel && cargo test -p ucel-core -p ucel-ir -p ucel-registry -p ucel-sdk` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_us_official_contract_matrix --test ir_us_official_identity_resolution --test ir_us_official_document_discovery --test ir_us_official_artifact_download --test ir_us_official_access_guard --test ir_us_official_docs_drift` OK
  - `cd ucel && cargo test -p ucel-testkit -- --nocapture` OK
- trace-index json.tool OK (`python -m json.tool docs/status/trace-index.json > /dev/null`).
- Secrets scan OK (`rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` on touched files).
- docsリンク存在チェック OK (new docs with `docs/` references resolved).

## 4) ★履歴確認の証拠
- `git log --oneline --decorate -n 50` and `git log --graph --oneline --decorate --all -n 80` were checked around `a5ce88f` and merge sequence (`7426a8a`, `07ab7c3`).
- Last-touch checks were taken for `ir_inventory.json`, `ucel-ir/src/lib.rs`, `ucel-sdk/src/ir.rs`, `registry/hub/registry.rs`, and `ucel/docs/ir/ir_canonical_model.md`.
- `git blame -w` was checked for `ir_inventory.json`, `ucel-core/src/ir.rs`, `ucel-sdk/src/ir.rs`, and `registry/hub/registry.rs` to confirm prior design ownership and no revert evidence around US canonical contracts.
- `git reflog -n 30`, `git branch -vv`, and `git log --merges --oneline -n 30` were reviewed; no prior US-official adapter module existed under `ucel-ir/src/us_official/*`.
- `git merge-base HEAD origin/master` and origin/master-based conflict commands failed because `origin/master` is not present in this environment.

### Design rationale
- US official adapter logic is localized under `us_official/*` to avoid JP/issuer-site scope creep and reduce merge conflict risk.
- 014A inventory US official source (`sec_edgar_submissions_api`) is wired with canonical operations for issuer resolve, filing list/detail, and artifact list/download.
- JSON/API-first + HTML fallback + attachment discovery/download policy is represented by descriptor access patterns and docs flow.
- Filing family mapping is explicitly fixed (annual/quarterly/current/proxy/registration-like/insider-like/other) with canonical family plus form-like source metadata retention.
- Access/politeness guard enforces policy class gate, user-agent discipline, retry/backoff, and attachment size limits.

### Additional completion actions
- Added US helper routes in registry and SDK so canonical IR surface is reachable without direct adapter coupling in higher layers.
- Added docs drift gates to fail CI when source matrix, filing-family mapping, or access/politeness naming drifts from code.
