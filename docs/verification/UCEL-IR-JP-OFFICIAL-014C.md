# UCEL-IR-JP-OFFICIAL-014C Verification

## 1) Changed files
- docs/specs/ucel/ir_jp_official_surface_v1.md
- docs/specs/ucel/ir_jp_official_fetch_policy_v1.md
- docs/specs/ucel/ir_jp_official_identity_mapping_v1.md
- docs/specs/ucel/ir_jp_official_artifact_policy_v1.md
- ucel/docs/ir/jp_official_source_matrix.md
- ucel/docs/ir/jp_official_fetch_flow.md
- ucel/docs/ir/jp_official_identity_mapping.md
- ucel/docs/ir/jp_official_artifact_matrix.md
- ucel/docs/ir/jp_official_access_and_politeness.md
- ucel/crates/ucel-ir/src/lib.rs
- ucel/crates/ucel-ir/src/jp_official/mod.rs
- ucel/crates/ucel-ir/src/jp_official/statutory.rs
- ucel/crates/ucel-ir/src/jp_official/timely.rs
- ucel/crates/ucel-ir/src/jp_official/identity.rs
- ucel/crates/ucel-ir/src/jp_official/document.rs
- ucel/crates/ucel-ir/src/jp_official/artifact.rs
- ucel/crates/ucel-ir/src/jp_official/fetch.rs
- ucel/crates/ucel-ir/src/jp_official/html.rs
- ucel/crates/ucel-ir/src/jp_official/feed.rs
- ucel/crates/ucel-ir/src/jp_official/download.rs
- ucel/crates/ucel-ir/src/jp_official/access.rs
- ucel/crates/ucel-ir/src/jp_official/errors.rs
- ucel/crates/ucel-sdk/src/ir.rs
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/src/ir_jp_official.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_identity_resolution.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_document_discovery.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_artifact_download.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_access_guard.rs
- ucel/crates/ucel-testkit/tests/ir_jp_official_docs_drift.rs
- ucel/examples/ir_jp_official_preview.rs
- ucel/fixtures/ir_jp_official/issuers.json
- ucel/fixtures/ir_jp_official/documents.json
- ucel/fixtures/ir_jp_official/artifacts.json
- ucel/fixtures/ir_jp_official/artifact_bytes.json
- docs/status/trace-index.json
- docs/verification/UCEL-IR-JP-OFFICIAL-014C.md

## 2) What/Why
This task implements JP official IR source adapters (statutory + timely) on top of 014A inventory and 014B canonical contracts. Source-specific behavior is localized in `ucel-ir/src/jp_official/*`, keeping generic contracts stable. The adapter now supports issuer resolve, document list/detail, artifact list/download, and fallback metadata paths while preserving provenance. Access/politeness controls were added as code guards (review-required and blocked fail-fast, plus retry/backoff/size caps). SDK and registry now expose JP-official reachability helpers and test gates pin behavior and docs alignment.

## 3) Self-check results
- Allowed-path check: OK (staged files only).
- Tests added/updated:
  - ir_jp_official_contract_matrix
  - ir_jp_official_identity_resolution
  - ir_jp_official_document_discovery
  - ir_jp_official_artifact_download
  - ir_jp_official_access_guard
  - ir_jp_official_docs_drift
- Build/Unit test command results:
  - `python -m json.tool ucel/coverage_v2/ir/ir_inventory.json > /dev/null` OK
  - `rg -n '"market": "jp"' ucel/coverage_v2/ir/ir_inventory.json` OK
  - `cd ucel && cargo test -p ucel-core -p ucel-ir -p ucel-registry -p ucel-sdk` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_contract_matrix -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_identity_resolution -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_document_discovery -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_artifact_download -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_access_guard -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_jp_official_docs_drift -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit` started but stopped due long-running full-suite runtime in this environment.
- trace-index json.tool OK (`python -m json.tool docs/status/trace-index.json > /dev/null`).
- Secrets scan OK (`rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` on touched files).
- docsリンク存在チェック OK (new docs with `docs/` references resolved).

## 4) ★履歴確認の証拠
- `git log --oneline --decorate -n 50` / `git log --graph --oneline --decorate --all -n 80` reviewed around `18257e3` (014B), `766b669` (014A), and prior series commits.
- `git show HEAD --stat` and last-touch checks executed for `ir_inventory.json`, `ucel-core/src/ir.rs`, `ucel-sdk/src/ir.rs`, `registry/hub/registry.rs`, `ucel-ir/src/lib.rs`.
- `git blame -w` reviewed for `ir_inventory.json`, `ucel-core/src/ir.rs`, `ucel-sdk/src/ir.rs`, `registry/hub/registry.rs`; logs reviewed for `ucel-ir` and `ucel/docs/ir` hotspots.
- `git reflog -n 30`, `git branch -vv`, `git log --merges --oneline -n 30` reviewed.
- `git merge-base HEAD origin/master` failed due missing `origin/master` ref in this environment.

### Design rationale
- JP official adapter/fallback logic is isolated under `jp_official/*` to avoid cross-market drift.
- 014A inventory JP official sources (`edinet_api_documents_v2`, `jp_tdnet_timely_html`) are both routed.
- API-first + feed/html fallback is represented in descriptor access patterns and flow docs.
- Access/politeness guard enforces review-required/default-deny and size/retry/backoff requirements.
- Artifact fetch success requires metadata presence (kind/content type/size/checksum/source URL).

### Additional completion actions
- Filled previously not_implemented timely route by implementing review-gated adapter path and tests.
- Added explicit JP helper paths in registry and SDK to avoid unresolved route drift.
