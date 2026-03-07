# UCEL-DOMESTIC-PUBLIC-COMPAT-009F Verification

## 1) Changed files (`git diff --name-only` target)
- docs/specs/ucel/domestic_public_compat_gate_v1.md
- docs/specs/ucel/domestic_public_schema_evolution_policy_v1.md
- ucel/docs/exchanges/domestic_public_final_support_report.md
- ucel/docs/exchanges/domestic_public_compat_matrix.md
- ucel/docs/exchanges/domestic_public_change_management.md
- ucel/coverage_v2/domestic_public/jp_public_inventory.lock.json
- ucel/crates/ucel-testkit/src/domestic_public_compat.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_inventory_lock.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_route_reachability.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_docs_and_matrices.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_fixtures_and_goldens.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_schema_runtime_evolution.rs
- ucel/crates/ucel-testkit/tests/domestic_public_compat_workspace_scope.rs
- ucel/examples/domestic_public_compat_preview.rs
- ucel/fixtures/domestic_public_goldens/domestic_public_rest_golden.json
- ucel/fixtures/domestic_public_goldens/domestic_public_ws_golden.json
- ucel/fixtures/domestic_public_goldens/domestic_public_ext_rest_golden.json
- ucel/fixtures/domestic_public_goldens/domestic_public_ext_ws_golden.json
- ucel/fixtures/domestic_public_goldens/domestic_public_inventory_golden.json
- scripts/check_domestic_public_lock.py
- docs/status/trace-index.json
- docs/verification/UCEL-DOMESTIC-PUBLIC-COMPAT-009F.md

## 2) What / Why
This task introduces a final domestic-public compatibility gate that binds inventory, lock snapshot, route reachability, docs/matrix summaries, fixture goldens, schema/runtime policy docs, and workspace venue scope. The lock file pins stable identifiers so count-only drift cannot pass. New tests are split by gate dimension to fail fast on any mismatch. Final support/report docs formalize that `partial/not_implemented` is zero. A standalone checker script was added for CI-friendly inventory-lock verification.

## 3) Self-check results
- Allowed-path check: OK (all staged paths in allowlist)
- Tests added/updated: 
  - domestic_public_compat_inventory_lock
  - domestic_public_compat_route_reachability
  - domestic_public_compat_docs_and_matrices
  - domestic_public_compat_fixtures_and_goldens
  - domestic_public_compat_schema_runtime_evolution
  - domestic_public_compat_workspace_scope
- Build / unit test command results:
  - `python -m json.tool ucel/coverage_v2/domestic_public/jp_public_inventory.json > /dev/null` OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` OK
  - `./scripts/check_domestic_public_lock.py` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_inventory_lock -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_route_reachability -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_docs_and_matrices -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_fixtures_and_goldens -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_schema_runtime_evolution -- --nocapture` OK
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_compat_workspace_scope -- --nocapture` OK
  - `cd ucel && cargo test --workspace` started but stopped due long-running full workspace compile in this environment.
- trace-index json.tool: OK
- Secrets scan: `rg -n "(AKIA|SECRET|TOKEN|PASSWORD|PRIVATE KEY)"` across changed files found no credential-like additions.
- docs link existence (docs/ references touched in this task): checked manually; all linked paths exist.

## 4) 履歴確認の証拠
- `git log --oneline --decorate -n 80`: reviewed, no revert markers around domestic-public series artifacts.
- `git log --graph --oneline --decorate --all -n 120`: reviewed branch topology and local branch creation.
- `git show HEAD --stat`: inspected current head baseline intent.
- last-touch / blame review executed for:
  - `ucel/coverage_v2/domestic_public/jp_public_inventory.json`
  - `ucel/crates/ucel-core/src/public_ws.rs`
  - `ucel/crates/ucel-core/src/public_rest_ext.rs`
  - `ucel/crates/ucel-core/src/public_ws_ext.rs`
  - `ucel/crates/ucel-registry/src/hub/rest.rs`
  - `ucel/crates/ucel-registry/src/hub/ws.rs`
  - `ucel/crates/ucel-sdk/src/public_ws.rs`
  - `ucel/crates/ucel-sdk/src/public_rest_ext.rs`
  - `ucel/crates/ucel-sdk/src/public_ws_ext.rs`
- `git log --oneline -n 5` reviewed for hotspots:
  - `ucel/crates/ucel-testkit`
  - `ucel/docs/exchanges`
  - `ucel/fixtures`
- `git reflog -n 30` reviewed local branch/checkout evidence.
- `git merge-base HEAD origin/master`: repository has no `origin/master` ref in this environment (command failed); noted as environment-specific limitation.
- `git branch -vv` and `git log --merges --oneline -n 30`: reviewed branch pointers and recent merges.

### 009A〜009E consistency conclusions
- Inventory/documented domestic venue scope is stable across reviewed files: bitbank/bitflyer/coincheck/gmocoin/bittrade/sbivc.
- route/evidence hotspot remains registry hub + SDK/public extension surfaces.
- final gate adds lock+tests without broad runtime refactor (conflict-minimized localization in `ucel-testkit`).
- inventory/lock/docs/routes/fixtures/schema/runtime are now linked by explicit tests.
- `partial/not_implemented` count verified as zero through compat gate tests and report.
- no additional missing route or doc drift remained after introducing lock and matrix checks.
