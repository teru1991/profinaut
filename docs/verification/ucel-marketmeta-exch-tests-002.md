# UCEL-MARKETMETA-EXCH-TESTS-002 Verification

## 1) Changed files (`git diff --name-only`)
- docs/status/trace-index.json
- docs/verification/ucel-marketmeta-exch-tests-002.md
- ucel/crates/ucel-cex-binance/src/symbols.rs
- ucel/crates/ucel-cex-binance-usdm/src/symbols.rs
- ucel/crates/ucel-cex-binance-coinm/src/symbols.rs
- ucel/crates/ucel-cex-binance-options/src/symbols.rs
- ucel/crates/ucel-cex-bybit/src/symbols.rs
- ucel/crates/ucel-cex-bitget/src/symbols.rs
- ucel/crates/ucel-cex-okx/src/symbols.rs
- ucel/crates/ucel-cex-kraken/src/symbols.rs
- ucel/crates/ucel-cex-htx/src/symbols.rs
- ucel/crates/ucel-cex-bittrade/src/symbols.rs
- ucel/crates/ucel-cex-gmocoin/src/symbols.rs
- ucel/crates/ucel-cex-bitmex/src/symbols.rs
- ucel/crates/ucel-cex-coinbase/src/symbols.rs
- ucel/crates/ucel-cex-deribit/src/symbols.rs

## 2) What / Why
- Added fixture-only unit tests across all targeted exchange symbol adapters to ensure DTO→map→Instrument paths fail fast when tick/step are missing or unparsable.
- Kept tests network-free by using `serde_json` fixtures and private mapping functions / test-only helpers.
- Added Bybit/Bitget/OKX/Kraken/HTX/Bittrade/GMO regression tests for missing field behavior and precision/scale mapping.
- Strengthened Binance spot/usdm/coinm/options tests (including MIN_NOTIONAL alias variants and options scale fallback).
- Corrected options scale fallback conversion to deterministic decimal step construction (`Decimal::new(1, scale)`) so fallback test is meaningful.

## 3) Self-check results
- Allowed-path check OK:
  - `git diff --name-only | awk '{ ok=($0 ~ /^docs\// || $0 ~ /^ucel\/crates\// || $0=="ucel/Cargo.toml" || $0=="ucel/Cargo.lock" || $0 ~ /^\.github\/workflows\// ); if(!ok) print $0 }'`
  - Result: no output
- Tests added/updated OK:
  - binance, binance-usdm, binance-coinm, binance-options, bybit, bitget, okx, kraken, htx, bittrade, gmocoin, bitmex, coinbase, deribit
- Build/Unit test results:
  - `cd ucel && cargo fmt --check` => PASS
  - `cd ucel && cargo test -p ucel-cex-binance -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-binance-usdm -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-binance-coinm -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-binance-options -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-bybit -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-bitget -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-okx -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-kraken -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-htx -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-bittrade -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-gmocoin -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-bitmex -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-coinbase -q` => PASS
  - `cd ucel && cargo test -p ucel-cex-deribit -q` => PASS
  - `cd ucel && cargo test --all-features -q` => PASS
- trace-index json.tool OK:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan:
  - `rg -n "AKIA|SECRET|TOKEN|BEGIN PRIVATE KEY" <changed-files>` => PASS (no hits)
- docsリンク存在チェック:
  - changed docs paths (`docs/status/trace-index.json`, `docs/verification/ucel-marketmeta-exch-tests-002.md`) exist => OK

## 4) History evidence (required 0.1)
- `git log --oneline --decorate -n 50`
  - Head baseline: `4429a0cf feat(ucel): public REST Snapshot→MarketMeta ...`.
  - Conclusion: this task is stacked directly on prior Task1 implementation commit.
- `git log --graph --oneline --decorate --all -n 80`
  - Shows task branch from `work` at `4429a0cf`, stash refs captured during preflight cleanup.
  - Conclusion: clean branch sequencing for test-only follow-up.
- `git log --merges --oneline -n 30`
  - Recent merge chain includes #404, #403, #402...
  - Conclusion: repository history consistent with prior marketmeta rollout.
- `git reflog -n 30`
  - `HEAD@{0}`: checkout `work` -> `feature/ucel-marketmeta-exch-tests-002`.
  - Conclusion: task branch created as requested.
- `git merge-base HEAD origin/<default-branch>`
  - Remote default-branch merge-base unavailable in this environment (`origin/master` not resolvable).
  - Conclusion: used local `work` tip as effective base.
- `git blame -w ucel/crates/<crate>/src/symbols.rs`
  - Verified provenance of target adapters before editing; key recent SHAs include `4429a0cf`, `c359fa4e`, `f16368b5`, `efb302ae`.
  - Conclusion: edits are localized additive test hardening around existing mapping logic.
