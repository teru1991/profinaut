# UCEL-DECIMAL-CODE-CEX-003 Verification

## 1) Changed files
- docs/status/trace-index.json
- docs/verification/UCEL-DECIMAL-CODE-CEX-003.md
- ucel/crates/ucel-cex-coinbase/src/lib.rs
- ucel/crates/ucel-cex-upbit/src/lib.rs
- ucel/crates/ucel-cex-binance-options/src/lib.rs
- ucel/crates/ucel-cex-deribit/src/lib.rs
- ucel/crates/ucel-cex-sbivc/src/lib.rs
- ucel/crates/ucel-cex-bitbank/src/lib.rs
- ucel/crates/ucel-cex-binance-coinm/src/lib.rs
- ucel/crates/ucel-cex-htx/src/lib.rs
- ucel/crates/ucel-cex-kraken/src/lib.rs

## 2) What / Why
- Applied Decimal guard/serde policy on CEX connector wire inputs for price/qty-like values to reduce unchecked input risk.
- Replaced remaining `f64` trade/ticker value fields with `Decimal` in target connector wire/event types.
- Strengthened string-to-Decimal parse paths (bitbank/binance-coinm) with negative-value rejection for safer defaults.
- Added guarded deserialization helpers for optional/nested numeric payloads in Upbit/Deribit/HTX where direct field attributes were insufficient.
- Made Kraken `add_order` payload pass through `OrderGate` quantize/validate and send decimal strings for final tick/step enforcement at order boundary.

## 3) Self-check results
- Allowed-path check OK
  - Only `docs/**` and `ucel/crates/**` changed.
- Tests added/updated OK
  - No new test files; existing connector tests executed where build graph allowed.
- Build/Unit test command results
  - `cargo test -p ucel-cex-coinbase` => FAIL (blocked by pre-existing compile errors in `ucel-transport`: undefined `pen`/`prio`)
  - `cargo test -p ucel-cex-upbit` => FAIL (same blocker)
  - `cargo test -p ucel-cex-binance-options` => FAIL (same blocker)
  - `cargo test -p ucel-cex-deribit` => FAIL (same blocker)
  - `cargo test -p ucel-cex-sbivc` => PASS
  - `cargo test -p ucel-cex-bitbank` => FAIL (same blocker)
  - `cargo test -p ucel-cex-binance-coinm` => FAIL (same blocker)
  - `cargo test -p ucel-cex-htx` => FAIL (same blocker)
  - `cargo test -p ucel-cex-kraken` => FAIL (same blocker)
  - `cargo test -p ucel-core -p ucel-symbol-core -p ucel-cex-coinbase -p ucel-cex-upbit -p ucel-cex-binance-options -p ucel-cex-deribit -p ucel-cex-sbivc -p ucel-cex-bitbank -p ucel-cex-binance-coinm -p ucel-cex-htx -p ucel-cex-kraken` => FAIL (same blocker)
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan OK
  - Changed files scanned for key-like patterns => PASS
- docsリンク存在チェック OK
  - Added docs references from trace-index diff exist => PASS
