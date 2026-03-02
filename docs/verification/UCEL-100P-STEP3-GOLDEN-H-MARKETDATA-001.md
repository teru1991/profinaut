# UCEL-100P-STEP3-GOLDEN-H-MARKETDATA-001 Verification

## 1) Changed files
- ucel/crates/ucel-testkit/src/golden.rs
- ucel/crates/ucel-testkit/src/fixtures.rs
- ucel/crates/ucel-testkit/src/normalize.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/golden_ws.rs
- ucel/fixtures/golden/ws/bithumb/trade_snapshot/raw.json
- ucel/fixtures/golden/ws/bithumb/trade_snapshot/expected.normalized.json
- ucel/fixtures/golden/ws/bithumb/ticker_snapshot/raw.json
- ucel/fixtures/golden/ws/bithumb/ticker_snapshot/expected.normalized.json
- docs/status/trace-index.json
- docs/verification/UCEL-100P-STEP3-GOLDEN-H-MARKETDATA-001.md

## 2) What / Why
- Added a reusable market-data golden harness in `ucel-testkit` to load fixtures, dispatch venue decoders, canonicalize output, and compare deterministically.
- Added fixture discovery rules that enforce the existence of `raw.(json|txt)` and `expected.normalized.json` per case.
- Added deterministic canonicalization logic (stable object key order + stable event-array sorting for trade-like records while preserving orderbook ladder semantics).
- Added a strict-venue-wide test (`golden_ws.rs`) that reads `ucel/coverage/*.yaml`, discovers `strict=true` venues, and validates all WS golden cases.
- Added minimal public Bithumb WS golden cases for trade and ticker snapshots as CI proof fixtures.

## 3) Self-check results
- Allowed-path check: task changes are only under allowed paths (repo has unrelated pre-existing dirty files outside scope).
- Tests:
  - `cargo test -p ucel-testkit --manifest-path ucel/Cargo.toml --test golden_ws` ✅
  - `cargo test -p ucel-testkit --manifest-path ucel/Cargo.toml --test contract_ws_bithumb` ✅
  - `cargo test --manifest-path ucel/Cargo.toml --workspace --all-targets` ⚠️ (pre-existing `ucel-cex-sbivc` unresolved types)
- Golden coverage:
  - strict venues discovered from coverage: `bithumb` (1 venue)
  - fixture cases validated for strict venue: `bithumb/trade_snapshot`, `bithumb/ticker_snapshot` (2 cases)
- Secrets scan: fixtures contain only public market-data samples (no keys/tokens/private account data) ✅

## 4) 履歴確認の証拠（0.1要点）
- `git log --oneline --decorate -n 50`: latest chain includes Step2 commit and earlier SSOT coverage-v1 merge lineage; no conflicting policy reversal observed.
- `git log --graph --oneline --decorate --all -n 80`: confirms this branch is stacked from Step2 branch and follows existing SSOT migration flow.
- `git show HEAD`: verifies the immediate parent commit intent (`feat(ucel): implement Bithumb public adapter and promote strict coverage`).
- `git reflog -n 30`: confirms branch creation from `feature/ucel-100p-step2-strict-100p-bithumb-001`.
- `git merge-base HEAD work`: confirms shared base with current `work` lineage.
- `git branch -vv` / `git log --merges --oneline -n 30`: no unexpected merge detours against recent UCEL SSOT work.
- Existing golden baseline evidence:
  - `ls ucel/fixtures/golden/ws` shows venues: `bithumb`, `bybit`.
  - `git blame -w ucel/crates/ucel-testkit/src/golden.rs` and `git blame -w ucel/crates/ucel-testkit/tests/golden_ws_*.rs` confirm the prior bybit golden pattern this task extends rather than replaces.

## 5) Step4への引き継ぎ（fuzz候補）
- Fuzz candidate #1: Bithumb WS `content` field shape variance (object vs array) to ensure graceful unknown-path handling.
- Fuzz candidate #2: Decimal boundary strings (`0`, very long precision, scientific notation) across trade/ticker parsing.
- Fuzz candidate #3: Event-array nondeterminism under mixed ordering (`trade_id` collisions / missing sortable keys).
- Fuzz candidate #4: malformed envelope fixtures (`missing endpoint_id`, invalid payload type) in fixture loader.
