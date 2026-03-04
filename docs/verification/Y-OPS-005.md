# Y-OPS-005 Verification

## 1) Changed files
- docs/runbooks/y_support_bundle_runbook.md
- docs/status/trace-index.json
- ucel/Cargo.toml
- ucel/Cargo.lock
- ucel/crates/ucel-diagnostics-cli/Cargo.toml
- ucel/crates/ucel-diagnostics-cli/src/main.rs
- ucel/crates/ucel-diagnostics-cli/src/args.rs
- ucel/crates/ucel-diagnostics-cli/src/rbac.rs
- ucel/crates/ucel-diagnostics-cli/src/audit.rs
- ucel/crates/ucel-diagnostics-cli/src/export.rs
- ucel/crates/ucel-diagnostics-cli/tests/triage_smoke.rs
- ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs
- docs/verification/Y-OPS-005.md

## 2) What / Why
- Added `ucel-diagnostics-cli` to provide reproducible Y-domain operational flow via `analyze`, `export`, and `whoami` commands.
- `Analyze` is read-only and always routes bundle parsing/integrity/diag_semver checks through `ucel-diagnostics-analyzer` before writing `summary.json`.
- `Export` enforces break-glass controls (TTL, reason, approvals threshold) and always emits encrypted output (XChaCha20-Poly1305 JSON envelope).
- Added audit JSONL chain-hash recording (`prev_hash_hex`/`this_hash_hex`) for both success and failure paths.
- Added Y runbook documenting one-command triage and guarded export flow; added transport-side minimal audit hook call-point for support bundle archive requests.

## 3) Self-check results
- Allowed-path check: OK
- Tests added/updated:
  - `ucel/crates/ucel-diagnostics-cli/tests/triage_smoke.rs`
- Build/Test commands:
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-diagnostics-cli` => OK
- trace-index json.tool:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => OK
- Secrets scan:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" ucel/crates/ucel-diagnostics-cli docs/runbooks/y_support_bundle_runbook.md -S || true`
  - 結果: 実データの漏洩はなし。説明文中の一般語（`secret-free` や `token`）のみヒット。
- Preflight caveat:
  - repository had unrelated untracked dirs (`apps/web/node_modules/`, `tools/validate_schemas/node_modules/`) before work; task scope avoided them.

## 4) History evidence (required)
- `git log --oneline --decorate -n 50`
  - Latest chain confirms Y diagnostics progression: `fcd402a` (Y-CORE-001), `e271ce7` (Y-BUNDLE-002), `73b2c82` (Y-REDACT-003), `cae82e9` (Y-ANALYZER-004 baseline).
- `git log --graph --oneline --decorate --all -n 80`
  - Confirms Y-area landed in staged PRs and this task sits on top of analyzer baseline for operational completion.
- `git log --merges --oneline -n 30`
  - Confirms merge trajectory around diagnostics contract establishment (#449/#450/#451).
- `git reflog -n 30`
  - Branch transition: `feature/y-analyzer-004` -> `feature/y-ops-005` at `cae82e9`.
- `git merge-base HEAD origin/<default-branch>`
  - Not resolvable in this environment (`origin/master` missing). Logged as environment limitation.
- `git branch -vv`
  - Active branch `feature/y-ops-005` based on `cae82e9`; `work` branch remains at `c78259d`.
- `rg -n "rbac|role|policy|break.?glass|approval|audit|triage|support_bundle" ...`
  - Found support-bundle and audit policy references but no complete diagnostics CLI path providing combined RBAC+break-glass+audit+export control.
- `git log --oneline -n 30 -- docs/runbooks`
  - Runbooks are active and expected to encode operational contract text.
- `git log --oneline -n 30 -- ucel/crates/ucel-transport/src/diagnostics`
  - Existing diagnostics bundle implementation present; no prior explicit archive-request audit hook.
- `git blame -w docs/specs/system/Y_Supportability_Diagnostics_Governance_Spec_v1.0.md | head -n 220`
  - Governance spec emphasizes approval_id/break-glass constraints and audit-centric operations.
- `git blame -w docs/specs/crosscut/support_bundle_spec.md | head -n 220`
  - Support bundle remains manifest-first, secret-free, audit-linked; implementation must stay fail-closed and traceable.

## 5) Conclusion
- Break-glass enforcement, tamper-evident audit output, encrypted export path, and reproducible triage command path were incomplete as one integrated operator workflow.
- This task closes that operational gap with concrete code paths and tests while preserving existing support bundle contracts.
