# Verification: UCEL-TESTKIT-PROOF-GOLDEN-001

## 1) Changed files
```bash
git diff --name-only
```

- docs/status/trace-index.json
- docs/verification/UCEL-TESTKIT-PROOF-GOLDEN-001.md
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/src/golden.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/golden_ws_bybit.rs
- ucel/fixtures/golden/ws/bybit/expected.normalized.json
- ucel/fixtures/golden/ws/bybit/raw.json

## 2) What / Why (3-7 lines)

Implemented a new additive golden harness in `ucel-testkit` for fixture loading, canonicalized JSON comparison, and first-diff-path reporting.

Added Bybit websocket golden fixtures (`raw.json` and `expected.normalized.json`) under `ucel/fixtures/golden/ws/bybit`.

Added a proof test that loads the fixture, runs `ucel_cex_bybit::normalize_ws_event`, and performs strict canonical JSON equality.

Changes are intentionally localized: one new module, one export line in `lib.rs`, one dependency line in `Cargo.toml`, one test file, and fixture/docs additions.

## 3) Self-check results

Allowed-path check OK:

```bash
git diff --name-only | awk '{
  ok=($0 ~ /^docs\// || $0 ~ /^ucel\// || $0=="Cargo.toml" || $0=="Cargo.lock" || $0 ~ /^tests\// || $0 ~ /^scripts\// || $0 ~ /^\.github\/workflows\//);
  if(!ok) print $0
}'
```

Output included pre-existing unrelated local change: `services/marketdata-rs/Cargo.lock` (already dirty before task).

New task files all remain within allowlist paths.

Tests:

```bash
cd ucel && cargo test -p ucel-testkit --test golden_ws_bybit
cd ucel && cargo test -p ucel-testkit
cd ucel && cargo test
```

All passed in this branch.

trace-index json.tool OK:

```bash
python -m json.tool docs/status/trace-index.json > /dev/null
```

Secrets scan (quick):

```bash
git diff | grep -Ei '(api[_-]?key|secret|token|password|Bearer )' || true
```

No secrets found.

## 4) ★履歴確認の証拠（必須）

Executed repository-history checks before implementation:

```bash
git log --oneline --decorate -n 50
git log --graph --oneline --decorate --all -n 80
git show --stat --oneline HEAD
git reflog -n 30
git branch -vv
git log --merges --oneline -n 30
```

`git merge-base HEAD origin/master` was attempted and failed because this clone has no configured remote-tracking refs (`fatal: Not a valid object name origin/master`).

Target-file history checks were run:

```bash
git log -n 20 -- ucel/crates/ucel-testkit/src/lib.rs
git show fcdde1b6dbc39377f3efda7ffb87a3cd7dab1559 -- ucel/crates/ucel-testkit/src/lib.rs
git blame -w ucel/crates/ucel-testkit/src/lib.rs

git log -n 20 -- ucel/crates/ucel-testkit/Cargo.toml
git show fcdde1b6dbc39377f3efda7ffb87a3cd7dab1559 -- ucel/crates/ucel-testkit/Cargo.toml
git blame -w ucel/crates/ucel-testkit/Cargo.toml
```

Summary of intent from history/blame:
- `ucel-testkit` evolves additively (new modules/exports and gate helpers) without altering runtime behavior.
- `Cargo.toml` in testkit is a focused path-dependency list for local UCEL crates.
- This task follows the same additive pattern and introduces no runtime path changes outside tests/fixtures.

Conclusion: golden tests are additive and do not change runtime behavior.
