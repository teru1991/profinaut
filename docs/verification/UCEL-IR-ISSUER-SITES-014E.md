# UCEL-IR-ISSUER-SITES-014E Verification

## 1) Changed files
- docs/specs/ucel/ir_issuer_sites_surface_v1.md
- docs/specs/ucel/ir_issuer_sites_discovery_policy_v1.md
- docs/specs/ucel/ir_issuer_sites_fetch_policy_v1.md
- docs/specs/ucel/ir_issuer_sites_profile_schema_v1.md
- docs/specs/ucel/ir_issuer_sites_artifact_policy_v1.md
- ucel/docs/ir/jp_us_issuer_site_matrix.md
- ucel/docs/ir/issuer_site_discovery_flow.md
- ucel/docs/ir/issuer_site_profile_guide.md
- ucel/docs/ir/issuer_site_artifact_matrix.md
- ucel/docs/ir/issuer_site_access_and_politeness.md
- ucel/coverage_v2/ir/ir_inventory.json
- ucel/crates/ucel-ir/src/lib.rs
- ucel/crates/ucel-ir/src/issuer_sites/mod.rs
- ucel/crates/ucel-ir/src/issuer_sites/jp.rs
- ucel/crates/ucel-ir/src/issuer_sites/us.rs
- ucel/crates/ucel-ir/src/issuer_sites/discovery.rs
- ucel/crates/ucel-ir/src/issuer_sites/profile.rs
- ucel/crates/ucel-ir/src/issuer_sites/identity.rs
- ucel/crates/ucel-ir/src/issuer_sites/document.rs
- ucel/crates/ucel-ir/src/issuer_sites/artifact.rs
- ucel/crates/ucel-ir/src/issuer_sites/fetch.rs
- ucel/crates/ucel-ir/src/issuer_sites/html.rs
- ucel/crates/ucel-ir/src/issuer_sites/feed.rs
- ucel/crates/ucel-ir/src/issuer_sites/download.rs
- ucel/crates/ucel-ir/src/issuer_sites/access.rs
- ucel/crates/ucel-ir/src/issuer_sites/errors.rs
- ucel/crates/ucel-sdk/src/ir.rs
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/src/ir_issuer_sites.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_discovery.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_identity_resolution.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_document_discovery.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_artifact_download.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_access_guard.rs
- ucel/crates/ucel-testkit/tests/ir_issuer_sites_docs_drift.rs
- ucel/examples/ir_issuer_sites_preview.rs
- ucel/fixtures/ir_issuer_sites/seeds.json
- ucel/fixtures/ir_issuer_sites/issuers.json
- ucel/fixtures/ir_issuer_sites/documents.json
- ucel/fixtures/ir_issuer_sites/artifacts.json
- ucel/fixtures/ir_issuer_sites/artifact_bytes.json
- docs/status/trace-index.json
- docs/verification/UCEL-IR-ISSUER-SITES-014E.md

## 2) What/Why
This task adds issuer-site IR adapters for JP/US source families on canonical UCEL IR contracts, with deterministic discovery and profile-driven extraction.
Implementation is localized under `ucel-ir/src/issuer_sites/*` to avoid cross-task conflicts with official-source modules.
Issuer-site flows now support issuer resolution, document list/detail, artifact list/download, and source/referring-page provenance.
Access and politeness are fixed as code guards (review-required default deny, blocked fail-fast, crawl/page budget, attachment size guard).
SDK and registry are wired with issuer-site routes/helpers, and new testkit gates lock matrix/discovery/identity/document/artifact/docs drift.

## 3) Self-check results
- Allowed-path check: OK (staged files only).
- Tests added/updated:
  - ir_issuer_sites_contract_matrix
  - ir_issuer_sites_discovery
  - ir_issuer_sites_identity_resolution
  - ir_issuer_sites_document_discovery
  - ir_issuer_sites_artifact_download
  - ir_issuer_sites_access_guard
  - ir_issuer_sites_docs_drift
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-core -p ucel-ir -p ucel-registry -p ucel-sdk` OK
  - `cd ucel && cargo test -p ucel-testkit --test ir_issuer_sites_contract_matrix --test ir_issuer_sites_discovery --test ir_issuer_sites_identity_resolution --test ir_issuer_sites_document_discovery --test ir_issuer_sites_artifact_download --test ir_issuer_sites_access_guard --test ir_issuer_sites_docs_drift` OK
  - `cd ucel && cargo test -p ucel-testkit` started and had to be interrupted due long-running suite in this environment.
- trace-index json.tool OK (`python -m json.tool docs/status/trace-index.json > /dev/null`).
- inventory json.tool OK (`python -m json.tool ucel/coverage_v2/ir/ir_inventory.json > /dev/null`).
- Secrets scan OK (`rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` on touched files).
- docsリンク存在チェック OK (new docs/spec files with `docs/` references resolved).

## 4) ★履歴確認の証拠
- `git log --oneline --decorate -n 50` and `git log --graph --oneline --decorate --all -n 80` reviewed around latest base commit `1497e45`, plus merges `7426a8a`, `07ab7c3`.
- `git show HEAD --stat` reviewed for immediate baseline.
- `git blame -w` reviewed for `ucel/coverage_v2/ir/ir_inventory.json` and `ucel/crates/ucel-core/src/ir.rs`.
- `git reflog -n 30`, `git branch -vv`, `git log --merges --oneline -n 30` reviewed.
- `git merge-base HEAD origin/master` and origin/master scoped logs failed in this environment because `origin/master` ref is missing.

### Design rationale
- Issuer site adapters are isolated under `issuer_sites/*` and exported via `ucel-ir/src/lib.rs` only.
- Discovery follows strict order: official metadata seed -> inventory seed -> deterministic traversal; no search-engine path is implemented.
- Site profile schema includes root/index/feed/selectors/attachment-rule and traversal limits for bounded HTML/feed extraction.
- Identity binding always returns provenance and blocks ambiguous/no-match mappings.
- Artifact fetch enforces content type/size guards and returns checksum/size/source_url/referring_page metadata.

### 014A/014B/014C/014D alignment
- Inventory JP/US issuer-site sources are routed via adapter map and SDK route table.
- Canonical contracts (`IrSourceAdapter`, document/artifact/identity envelopes) remain unchanged; issuer-site implementation conforms to existing canonical surface.
- Issuer-site statuses in inventory were updated to `implemented` for the four issuer-site source/document rows to match this task completion.

### Additional completion actions
- Added registry issuer-site helper readers to support SDK/testkit route checks.
- Added dedicated docs drift gate to keep matrix/flow/profile/artifact/access docs in sync with code terms.
