# UCEL-100P-STEP4-FUZZ-T-CRASHFREE-001 Verification

## Goal
- Add deterministic lightweight fuzz coverage for WS frame and JSON-depth inputs in PR CI.
- Prove malformed/oversized/deep inputs return `Err` and do not panic.
- Fix iOS PR instability by documenting Safari(Web) as the official PR creation path for this task class.

## Changed files
- `ucel/crates/ucel-testkit/src/lib.rs`
- `ucel/crates/ucel-testkit/src/fuzz.rs`
- `ucel/crates/ucel-testkit/src/fuzz_corpus.rs`
- `ucel/crates/ucel-testkit/tests/fuzz_ws_frames.rs`
- `ucel/crates/ucel-testkit/tests/fuzz_json_depth.rs`
- `ucel/fixtures/fuzz/ws_frames/small_valid_trade_frame.txt`
- `ucel/fixtures/fuzz/ws_frames/random_ascii_bytes.txt`
- `ucel/fixtures/fuzz/ws_frames/boundary_seed_repeatable.txt`
- `ucel/fixtures/fuzz/json/valid_small.json`
- `ucel/fixtures/fuzz/json/deep_nested.json`
- `ucel/fixtures/fuzz/json/wide_object.json`
- `docs/verification/UCEL-100P-STEP4-FUZZ-T-CRASHFREE-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git log --oneline --decorate -n 50`: captured current branch history snapshot.
- `git log --graph --oneline --decorate --all -n 80`: captured graph view for lineage.
- `git show HEAD`: captured HEAD summary as baseline.
- `git reflog -n 30`: captured recent local branch/ref operations.
- `git log --merges --oneline -n 30`: checked merge history.
- `git merge-base HEAD origin/main`: `origin/main` not available in this environment (no remote refs).
- `git branch -vv`: captured local branch tracking/commit pointers.
- `git blame -w ucel/crates/ucel-transport/src/ws/connection.rs`: sampled ownership of WS inbound guard path.

## Fuzz parameters
- RNG: xorshift64 deterministic seed `0xC0DEC0FFEE`
- Iterations: `N=200`
- `max_seed_bytes`: `256 * 1024`
- `max_generated_len`: `512 * 1024`
- Corpus folders:
  - `ucel/fixtures/fuzz/ws_frames/*.txt`
  - `ucel/fixtures/fuzz/json/*.json`

## Secrets scan
- Added fixtures are small text-only payloads (`.txt` / `.json`).
- No binary blobs and no credential-like material were intentionally introduced.

## iOS PR安定手順（正式手順）
1. iOS/PCで対象ブランチに切替し、まず Push を完了する。
2. PR作成は Safari(Web) を推奨（iOSアプリは差分性質により 400 失敗が起こり得る）。
3. PR本文テンプレ（5行以内）:
   - What: Add lightweight crash-free fuzz tests (seeded, deterministic)
   - Tests: cargo test --manifest-path ucel/Cargo.toml --workspace --all-targets
   - Verification: docs/verification/UCEL-100P-STEP4-FUZZ-T-CRASHFREE-001.md
