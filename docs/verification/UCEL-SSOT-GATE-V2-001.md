# Verification: UCEL-SSOT-GATE-V2-001

## 1) Changed files

`git diff --name-only` output at verification time:

```text
docs/specs/ucel/ssot_integrity_gate_v2.md
docs/status/trace-index.json
services/marketdata-rs/Cargo.lock
ucel/crates/ucel-registry/src/invoker/coverage.rs
ucel/crates/ucel-subscription-planner/src/lib.rs
ucel/crates/ucel-testkit/src/lib.rs
ucel/docs/ws-full-coverage-design.md
```

Note: `services/marketdata-rs/Cargo.lock` was pre-existing dirty state on branch and not part of this task.

## 2) What / Why (3-7 lines)

- Added v2 SSOT integrity policy doc defining “NOT SUPPORTED must be explicit” and effective strictness.
- Extended coverage v1 serde models with backward-compatible fields: `support` and `entry.strict`.
- Updated legacy coverage gate logic to ignore `support: not_supported` entries to prevent false failures.
- Added safe API (`extract_ws_ops_supported`, `ids_supported`) while preserving existing APIs.
- Added unit tests covering serde defaults, `not_supported` parsing, and filtering behavior.

## 3) Self-check results

Allowed-path check (staged-only) command:

```bash
git diff --name-only --cached | awk '{
  ok=($0 ~ /^docs\// || $0 ~ /^ucel\// || $0 ~ /^tests\// || $0 ~ /^scripts\// || $0=="Cargo.toml" || $0=="Cargo.lock" || $0 ~ /^\.github\/workflows\// );
  if(!ok) print $0
}'
```

Tests added/updated:

- `ucel/crates/ucel-subscription-planner/src/lib.rs` (`coverage_v1_tests`)

Build / Unit tests:

```bash
cd ucel
cargo test
```

Result summary: pass (`Finished test profile` and test suites complete with no failures).

trace-index json.tool check:

```bash
python -m json.tool docs/status/trace-index.json > /dev/null
```

Result: pass.

Secrets scan (quick):

```bash
git diff -- ucel docs | grep -Ei '(api[_-]?key|secret|token|password|Bearer )' || true
```

Result: no matches.

docs参照チェック:

```bash
grep -n "docs/" docs/specs/ucel/ssot_integrity_gate_v2.md ucel/docs/ws-full-coverage-design.md || true
```

Result: references listed and valid in-repo paths.

## 4) ★履歴確認の証拠（必須）

### 4.1 Repository history / branch topology

Commands run:

```bash
git log --oneline --decorate -n 50
git log --graph --oneline --decorate --all -n 80
git show --stat --oneline HEAD
git reflog -n 30
git branch -vv
git log --merges --oneline -n 30
git merge-base HEAD origin/master
```

Key observations:

- HEAD baseline before edits: `bf6c03d` (merge PR #407).
- Recent merges are linear merge-PR flow (`#407`..`#390`) with no conflicting intent on coverage v1 compatibility policy.
- No `origin/*` remote configured in this environment, so `git merge-base HEAD origin/master` is not executable here.

### 4.2 Target file history probes

Commands run per file:

```bash
git log -n 20 -- <file>
git show <last_touch_sha> -- <file>
git blame -w <file>
```

Highlights:

- `ucel/docs/ws-full-coverage-design.md` last touched by `d5cc3f8`; safety invariant wording already present and suitable for additive v2 paragraph.
- `ucel/crates/ucel-subscription-planner/src/lib.rs` has iterative WS-coverage evolution (`72aaa8e`, `16db767`, `0ffd77e`), so additive serde fields + new API match existing direction.
- `ucel/crates/ucel-testkit/src/lib.rs` history includes SSOT coverage gate task (`21614b9`), confirming this is the correct gate location for `not_supported` skip.
- `ucel/crates/ucel-registry/src/invoker/coverage.rs` introduced in `f9e6ee8`; additive `ids_supported` keeps backward compatibility with existing `ids`.
- `docs/status/trace-index.json` continuously extended (latest `312550e`), so task-key-only append follows existing maintenance pattern.

Conclusion: no conflicting intent found; this patch is additive and backward-compatible with v1 manifests.
