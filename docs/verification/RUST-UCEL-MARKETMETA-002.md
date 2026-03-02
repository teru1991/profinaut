# RUST-UCEL-MARKETMETA-002 Verification

## Scope
- Implemented MarketMetaFetcher for GMO Coin and Binance Spot.
- Added fixture-based parsing/normalization tests for tick/step/min_qty/max_qty/min_notional.
- Updated `docs/status/trace-index.json` task entry only for `RUST-UCEL-MARKETMETA-002`.

## Self-check
- Allowed-path OK: all final diffs are under `docs/**`, `ucel/crates/**`, `ucel/fixtures/**`, and `ucel/Cargo.lock`.
- Tests (fixture based, no network) added:
  - `ucel/crates/ucel-cex-gmocoin/tests/market_meta_parse_it.rs`
  - `ucel/crates/ucel-cex-binance/tests/market_meta_parse_it.rs`

## Commands and results
- `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check`
  - Result: failed initially due pre-existing formatting drift in `ucel-cex-upbit` (outside this task scope).
- `cargo fmt --manifest-path ucel/Cargo.toml --all`
  - Result: executed successfully.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-gmocoin -p ucel-cex-binance`
  - Result: pass.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-symbol-core -p ucel-symbol-store -p ucel-symbol-adapter -p ucel-sdk`
  - Result: pass.
- `python -m json.tool docs/status/trace-index.json > /dev/null`
  - Result: pass.
- `rg -n "(api[_-]?key|api[_-]?secret|token|secret)" ucel/fixtures/market_meta docs/verification/RUST-UCEL-MARKETMETA-002.md --ignore-case`
  - Result: no matches.

## Notes
- `MarketMeta.validate_meta()` is enforced during both venue mappings.
- Mapping/decimal/validation failures are returned as `MarketMetaAdapterError::Mapping`, and HTTP issues as `Transport`.
