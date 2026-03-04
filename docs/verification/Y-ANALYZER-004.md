# Verification: Y-ANALYZER-004

## 1) Changed files
- docs/runbooks/support_bundle_generation.md
- docs/runbooks/transport/ws_health_support_bundle.md
- docs/status/trace-index.json
- ucel/Cargo.toml
- ucel/Cargo.lock
- ucel/crates/ucel-diagnostics-analyzer/Cargo.toml
- ucel/crates/ucel-diagnostics-analyzer/src/lib.rs
- ucel/crates/ucel-diagnostics-analyzer/src/read.rs
- ucel/crates/ucel-diagnostics-analyzer/src/summary.rs
- ucel/crates/ucel-diagnostics-analyzer/src/synth.rs
- ucel/crates/ucel-diagnostics-analyzer/tests/synth_analyze.rs
- ucel/crates/ucel-diagnostics-analyzer/tests/compat_semver.rs
- ucel/crates/ucel-diagnostics-analyzer/tests/golden_external.rs
- ucel/crates/ucel-diagnostics-analyzer/tests/runbook_drift.rs
- ucel/fixtures/golden/support_bundle/v1/README.md
- ucel/fixtures/golden/support_bundle/v1/expected.summary.json

## 2) What / Why
- Added a new `ucel-diagnostics-analyzer` crate to read `tar.zst` support bundles, parse `manifest.json`, verify per-file integrity (`size_bytes` + `sha256`), and produce deterministic `summary` output.
- Added synthetic bundle generation for CI-safe regression testing without requiring binary fixture generation or updates.
- Added compatibility gating tests to ensure unsupported `diag_semver` major versions fail with explicit errors.
- Added runbook drift checks to fail if runbooks reference missing `docs/contracts/*.json` files, and to enforce `manifest.json` + `diag_semver` mentions in support-bundle-focused runbooks.
- Added optional external golden comparison test (`UCEL_GOLDEN_BUNDLE_DIR`) to validate manually supplied binary fixture(s) without making CI depend on binary updates.

## 3) Self-check results
- Allowed-path check: OK (no changed files outside allowlist).
- Tests: OK
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-diagnostics-analyzer`
- Golden policy:
  - Binary golden generation/update is not implemented.
  - Synthetic fixture is the always-on regression backbone.
  - External golden binary comparison is optional and env-guarded via `UCEL_GOLDEN_BUNDLE_DIR`.
- Secrets scan (manual grep-based sanity): no keys/tokens/real data introduced in this task’s new files.

## 4) History checks evidence (0.1)
- `git log --oneline --decorate -n 50`
  - HEAD baseline: `c78259d` (merge PR #451), with diagnostics contract lineage present: `fcd402a` (Y-CORE-001), `e271ce7` (Y-BUNDLE-002), `73b2c82` (Y-REDACT-003).
- `git log --graph --oneline --decorate --all -n 80`
  - Confirms diagnostic work landed incrementally through dedicated PR chain; analyzer implementation did not exist as a crate yet.
- `git log --merges --oneline -n 30`
  - Merge history shows dedicated diagnostics milestones and no prior analyzer merge.
- `git reflog -n 30`
  - Branch moved from `work` to `feature/y-analyzer-004` from same baseline commit `c78259d`.
- `git merge-base HEAD origin/<default-branch>`
  - `origin/<default-branch>` unavailable in this environment; fallback merge-base with local baseline branch `work` = `c78259d`.
- `git branch -vv`
  - Local branches: `feature/y-analyzer-004`, `work` (same SHA baseline).
- Existing analyzer/golden/runbook signals:
  - `rg -n "analyzer|summary\.json|golden bundle|runbook drift|support[_-]?bundle|diag_semver|Bundle" -S docs ucel/crates | head -n 80`
  - Existing support bundle + diag contracts were present, but no analyzer crate implementation in `ucel/crates`.
- `git log --oneline -n 30 -- docs/runbooks`
  - Runbooks are actively maintained and are part of operations flow.
- `git log --oneline -n 50 -- ucel/crates | rg -n "diagnostics|support_bundle|bundle|analyzer" -S`
  - Confirmed diagnostics/bundle core landed recently; analyzer still missing.
- `git blame -w docs/specs/crosscut/support_bundle_spec.md | head -n 200`
  - Confirms manifest-first, `diag_semver`, and fail-closed behavior are fixed constraints and should be enforced in analyzer/tests.

### Conclusions from history checks
- Analyzer implementation was missing/unfinished and safe to add as a new crate.
- Golden fixture policy should avoid binary generation in CI; synth + optional manual external fixture split is appropriate.
- Runbook references are operational contracts and should fail on dead links.

## 5) Manual binary fixture spec (if needed)
- Optional path: `ucel/fixtures/golden/support_bundle/v1/bundle_minimal.tar.zst`
- Required tar entries:
  1. `manifest.json`
  2. `meta/diag_semver.txt` = `1.0.0\n`
  3. `meta/info.json` = `{"k":"v"}`
  4. `logs/tail.txt` = `hello\n`
- `manifest.json` requirements:
  - must include bundle id, created-at timestamp, `diag_semver`, and `files` list
  - each listed file must have matching `size_bytes` and SHA-256 checksum (integrity gate)
- External test enablement:
  - `UCEL_GOLDEN_BUNDLE_DIR=/abs/path/to/ucel/fixtures/golden/support_bundle/v1 cargo test --manifest-path ucel/Cargo.toml -p ucel-diagnostics-analyzer --test golden_external`
